use lab::Lab;

use tiny_skia::*;

use rayon::prelude::*;
use std::sync::Mutex;
use std::time::Instant;

type ColorTable = Vec<Lab>;

mod draw_command;
mod renderer;

use crate::draw_command::DrawCommand;

#[inline(always)]
fn rgb2color(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16_u32) + ((g as u32) << 8_u32) + (b as u32)
}

fn diff(rgb2lab: &ColorTable, lab_img: &[Lab], canvas: &mut Canvas) -> i64 {
    let pixmap = canvas.pixmap();
    let w = pixmap.width();
    let h = pixmap.height();
    let img2: &[PremultipliedColorU8] = pixmap.pixels_mut();
    let mut sum = 0;
    for y in 0..h {
        for x in 0..w {
            let index = (x + w * y) as usize;
            let color2 = img2[index];
            let lab1 = lab_img[index];
            assert_eq!(color2.is_opaque(), true);
            let lab2 = rgb2lab[rgb2color(color2.red(), color2.green(), color2.blue()) as usize];

            let dl = lab1.l - lab2.l;
            let da = lab1.a - lab2.a;
            let db = lab1.b - lab2.b;
            let val = ((dl * dl + da * da + db * db) as f64).sqrt() as i64;
            sum += val as i64;
        }
    }
    sum
}

fn avg_color(w: u32, h: u32, img: &[u8]) -> ColorU8 {
    let mut sum_r = 0;
    let mut sum_g = 0;
    let mut sum_b = 0;
    for y in 0..h {
        for x in 0..w {
            let index2 = (x + w * y) as usize;
            let index = 4 * index2;
            let r = img[index] as u32;
            let g = img[index + 1] as u32;
            let b = img[index + 2] as u32;

            sum_r += r;
            sum_g += g;
            sum_b += b;
        }
    }
    let count = (w * h) as u32;
    if count == 0 {
        return ColorU8::from_rgba(0, 0, 0, 0xff);
    }
    let r = (sum_r / count) as u8;
    let g = (sum_g / count) as u8;
    let b = (sum_b / count) as u8;

    ColorU8::from_rgba(r, g, b, 0xff)
}

fn draw_cmd(canvas: &mut Canvas, cmd: &DrawCommand, is_antialias: bool) {
    renderer::fill_ellipse(
        canvas,
        cmd.x as f32,
        cmd.y as f32,
        cmd.rx as f32,
        cmd.ry as f32,
        cmd.angle as f32,
        &cmd.color,
        is_antialias,
    );
}

fn copy_img(src: &mut Canvas, dst: &mut Canvas) {
    let src_data = src.pixmap().data_mut();
    let dst_data = dst.pixmap().data_mut();
    assert_eq!(src_data.len(), dst_data.len());
    for i in 0..src_data.len() {
        dst_data[i] = src_data[i];
    }
}

fn try_draw(
    rgb2lab: &ColorTable,
    canvas: &mut Canvas,
    tmp_target: &mut Canvas,
    lab_img: &[Lab],
    cmd: &DrawCommand,
) -> i64 {
    copy_img(canvas, tmp_target);
    draw_cmd(tmp_target, &cmd, true);
    let score = diff(&rgb2lab, &lab_img, tmp_target);
    score
}

fn img2lab(rgb2lab: &ColorTable, w: u32, h: u32, img: &[u8]) -> Vec<Lab> {
    let mut result = Vec::new();

    for y in 0..h {
        for x in 0..w {
            let index2 = (x + w * y) as usize;
            let index = 4 * index2;
            let r = img[index];
            let g = img[index + 1];
            let b = img[index + 2];
            let color = rgb2color(r, g, b);
            let lab = rgb2lab[color as usize];
            result.push(lab);
        }
    }

    result
}

fn create_rgb2lab() -> ColorTable {
    let mut result = Vec::new();
    let lab0 = Lab::from_rgb(&[0, 0, 0]);
    result.resize(16777216, lab0);
    for r in 0..=255 {
        for g in 0..=255 {
            for b in 0..=255 {
                let color = rgb2color(r, g, b);
                let lab = Lab::from_rgb(&[r, g, b]);
                result[color as usize] = lab;
            }
        }
    }
    result
}

fn draw_bg(canvas: &mut Canvas, bg_color_string: &str, w: u32, h: u32, img: &[u8]) {
    println!("bg_color_string:{:?}", &bg_color_string);
    let bg_color;
    if bg_color_string == "avg" {
        bg_color = avg_color(w, h, &img);
    } else {
        let rgb = read_color::rgb(&mut bg_color_string.chars()).unwrap();
        bg_color = ColorU8::from_rgba(rgb[0], rgb[1], rgb[2], 0xff);
    }
    println!("bg_color:{:?}", &bg_color);
    let pixmap = canvas.pixmap();
    let w = pixmap.width();
    let h = pixmap.height();
    let mut paint = Paint::default();
    paint.set_color_rgba8(
        bg_color.red(),
        bg_color.green(),
        bg_color.blue(),
        bg_color.alpha(),
    );
    let rect = Rect::from_ltrb(0.0, 0.0, w as f32, h as f32).unwrap();
    canvas.fill_rect(rect, &paint);
}

fn canvas_from_vec(w: u32, h: u32, img_data: &mut Vec<u8>) -> Canvas {
    let pixmap = PixmapMut::from_bytes(img_data, w, h).unwrap();
    Canvas::from(pixmap)
}

fn vec_from_canvas(canvas_mutex: &Mutex<Canvas>) -> Vec<u8> {
    let mut canvas = canvas_mutex.lock().unwrap();
    let mut canvas_data: Vec<u8> = Vec::new();

    let data = canvas.pixmap().data_mut();

    for i in 0..data.len() {
        canvas_data.push(data[i]);
    }

    canvas_data
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = pico_args::Arguments::from_env();
    let path = args
        .opt_value_from_str(["--path", "-p"])?
        .unwrap_or("examples/monalisa_s.jpg".to_string());
    let num = args.opt_value_from_str(["--num", "-n"])?.unwrap_or(1000);
    let alpha = args.opt_value_from_str(["--alpha", "-a"])?.unwrap_or(128);
    let brush_scale = args
        .opt_value_from_str(["--brush-scale", "-b"])?
        .unwrap_or(0.75);
    let bg_color_string = args
        .opt_value_from_str(["--bg-color", "-bg"])?
        .unwrap_or("avg".to_string());
    let seed_count = args
        .opt_value_from_str(["--seed-count", "-s"])?
        .unwrap_or(32);
    let optimize_count = args
        .opt_value_from_str(["--optimize-count", "-o"])?
        .unwrap_or(64);

    let img = image::open(path).unwrap().to_rgba8();
    let w = img.width();
    let h = img.height();
    println!("{}x{}", w, h);
    let img_raw = img.into_raw();

    let rgb2lab = create_rgb2lab();
    let lab_img = img2lab(&rgb2lab, w, h, &img_raw);

    let mut global_best_score;

    let mut pixmap = Pixmap::new(w, h).unwrap();
    let canvas = Canvas::from(pixmap.as_mut());

    let canvas_mutex = Mutex::new(canvas);
    {
        let mut canvas = canvas_mutex.lock().unwrap();
        draw_bg(&mut canvas, &bg_color_string, w, h, &img_raw);
        global_best_score = diff(&rgb2lab, &lab_img, &mut canvas);
    }

    for t in 0..num {
        let t_ratio = (t as f32) / (num as f32);
        let start = Instant::now();

        let results: Vec<(i64, DrawCommand)> = (0..seed_count)
            .into_par_iter()
            .map(|_| {
                let mut rng = rand::thread_rng();

                let mut tmp_pixmap = Pixmap::new(w, h).unwrap();
                let mut tmp_target = Canvas::from(tmp_pixmap.as_mut());

                let mut canvas_data = vec_from_canvas(&canvas_mutex);

                let mut src_target = canvas_from_vec(w, h, &mut canvas_data);

                let mut best_cmd =
                    DrawCommand::rand(w, h, t_ratio, &img_raw, &mut rng, alpha, brush_scale);
                let mut best_score = try_draw(
                    &rgb2lab,
                    &mut src_target,
                    &mut tmp_target,
                    &lab_img,
                    &best_cmd,
                );

                // optimize
                for _j in 0..optimize_count {
                    let (cmd, cmd2) =
                        DrawCommand::mutate(w, h, t_ratio, &best_cmd, &mut rng, brush_scale);
                    let score;
                    if cmd == best_cmd {
                        score = best_score;
                    } else {
                        score =
                            try_draw(&rgb2lab, &mut src_target, &mut tmp_target, &lab_img, &cmd);
                    }
                    if score < best_score {
                        best_score = score;
                        best_cmd = cmd;
                    } else if cmd != cmd2 {
                        let score2;
                        if cmd2 == best_cmd {
                            score2 = best_score;
                        } else {
                            score2 = try_draw(
                                &rgb2lab,
                                &mut src_target,
                                &mut tmp_target,
                                &lab_img,
                                &cmd2,
                            );
                        }
                        if score2 < best_score {
                            best_score = score2;
                            best_cmd = cmd2;
                        }
                    }
                }

                (best_score, best_cmd)
            })
            .collect();

        let mut best_score = 0;
        let mut best_cmd = DrawCommand::new();
        for i in 0..results.len() {
            let (score, cmd) = results[i as usize];
            if i == 0 || score < best_score {
                best_score = score;
                best_cmd = cmd;
            }
        }

        let duration = start.elapsed();
        println!(
            "{} : {} {} {:?}",
            t, global_best_score, best_score, duration
        );

        if best_score < global_best_score {
            println!("   {:?}", &best_cmd);
            global_best_score = best_score;
            let mut canvas = canvas_mutex.lock().unwrap();
            //draw best cmd
            draw_cmd(&mut canvas, &best_cmd, true);
        }

        {
            let mut canvas = canvas_mutex.lock().unwrap();
            let img_name = format!("result_{:06}.png", t);
            canvas.pixmap().to_owned().save_png(img_name).unwrap();
        }
    }

    {
        let mut canvas = canvas_mutex.lock().unwrap();
        canvas.pixmap().to_owned().save_png("out.png").unwrap();
    }

    Ok(())
}

/*
todo


paint changed location in alpha white -> score weight
write svg check size
parse outputpath

resize in calc, save in original size

ssim image differnce
 https://en.wikipedia.org/wiki/Structural_similarity

write command
optimize svg
 quantize color(websafe), size
 two pass
 global optimize
  remove unneeded cmd
  mutate cmd

*/
