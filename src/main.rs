use euclid::{Angle, Size2D};
use lab::Lab;
use rand::distributions::{Distribution, Uniform};
use raqote::*;
use rayon::prelude::*;
use std::sync::Mutex;
use std::time::Instant;

type Color = SolidSource;

type ColorTable = Vec<Lab>;

const PI: f32 = std::f32::consts::PI;
const PI2: f32 = 2.0_f32 * PI;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawCommand {
    x: u32,
    y: u32,
    rx: u32,
    ry: u32,
    angle: i32, //degree
    color: Color,
}

fn rnd<T>(rng: &mut rand::rngs::ThreadRng, min: T, max: T) -> T
where
    T: rand::distributions::uniform::SampleUniform,
{
    let range = Uniform::new_inclusive(min, max);
    range.sample(rng)
}

fn clamp<T>(value: T, min: T, max: T) -> T
where
    T: std::cmp::PartialOrd,
{
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

fn mod_angle(angle: i32) -> i32 {
    let mut res = angle;
    if res < -90 {
        res += 180;
    }
    if angle > 90 {
        res -= 180;
    }
    res
}

fn brush_size(t_ratio: f32, brush_scale: f32, image_size: u32) -> u32 {
    let t1 = brush_scale * (1.0 - t_ratio);
    let t2 = t1 * t1;
    let size_ratio = t2;
    let size = 1 + (size_ratio * 0.5 * (image_size as f32)) as u32;
    size
}

impl DrawCommand {
    pub fn new() -> DrawCommand {
        let color = Color {
            r: 0,
            g: 0,
            b: 0,
            a: 0xff,
        };
        DrawCommand {
            x: 0,
            y: 0,
            rx: 0,
            ry: 0,
            angle: 0,
            color,
        }
    }

    pub fn rand(
        w: u32,
        h: u32,
        t_ratio: f32,
        img: &[u8],
        rng: &mut rand::rngs::ThreadRng,
        alpha: u8,
        brush_scale: f32,
    ) -> DrawCommand {
        let w1 = w - 1;
        let h1 = h - 1;
        let x = rnd(rng, 0, w1);
        let y = rnd(rng, 0, h1);

        let index = 4 * (x + w * y) as usize;
        let r = img[index];
        let g = img[index + 1];
        let b = img[index + 2];

        let rx_max = brush_size(t_ratio, brush_scale, w1);
        let ry_max = brush_size(t_ratio, brush_scale, h1);
        let rx_min = 1 + rx_max / 16;
        let ry_min = 1 + ry_max / 16;
        let rx = rnd(rng, rx_min, rx_max);
        let ry = rnd(rng, ry_min, ry_max);
        let angle = rnd(rng, -90, 90);
        let color = Color {
            r: r,
            g: g,
            b: b,
            a: alpha,
        };
        DrawCommand {
            x,
            y,
            rx,
            ry,
            angle,
            color,
        }
    }

    pub fn mutate(
        w: u32,
        h: u32,
        t_ratio: f32,
        original_cmd: &DrawCommand,
        rng: &mut rand::rngs::ThreadRng,
        brush_scale: f32,
    ) -> (DrawCommand, DrawCommand) {
        let w1 = w - 1;
        let h1 = h - 1;

        let mut cmd1 = original_cmd.clone();
        //inverse command
        let mut cmd2 = original_cmd.clone();

        let prop = rnd(rng, 0, 7) as u8;
        if prop == 0 {
            let dx_max = 2 + (w1 / 100);
            let dx = rnd(rng, 1, dx_max);
            cmd1.x = clamp(cmd1.x + dx, 0, w1);
            cmd2.x = clamp(cmd2.x - dx, 0, w1);
        } else if prop == 1 {
            let dy_max = 2 + (h1 / 100);
            let dy = rnd(rng, 1, dy_max);
            cmd1.y = clamp(cmd1.y + dy, 0, h1);
            cmd2.y = clamp(cmd2.y - dy, 0, h1);
        } else if prop == 2 {
            let dx_max = brush_size(t_ratio, brush_scale, w1);
            let dx = rnd(rng, 1, dx_max);
            cmd1.rx = clamp(cmd1.rx + dx, 1, w1 / 2);
            cmd2.rx = clamp(cmd2.rx - dx, 1, w1 / 2);
        } else if prop == 3 {
            let dy_max = brush_size(t_ratio, brush_scale, h1);
            let dy = rnd(rng, 1, dy_max);
            cmd1.ry = clamp(cmd1.ry + dy, 1, h1 / 2);
            cmd2.ry = clamp(cmd2.ry - dy, 1, h1 / 2);
        } else if prop == 4 {
            let d = rnd(rng, 1, 4);
            cmd1.angle = mod_angle(cmd1.angle + d);
            cmd2.angle = mod_angle(cmd2.angle - d);
        } else if prop == 5 {
            let d = rnd(rng, 1, 8);
            cmd1.color.r = clamp(cmd1.color.r + d, 0, 255) as u8;
            cmd2.color.r = clamp(cmd2.color.r - d, 0, 255) as u8;
        } else if prop == 6 {
            let d = rnd(rng, 1, 8);
            cmd1.color.g = clamp(cmd1.color.g + d, 0, 255) as u8;
            cmd2.color.g = clamp(cmd2.color.g - d, 0, 255) as u8;
        } else if prop == 7 {
            let d = rnd(rng, 1, 8);
            cmd1.color.b = clamp(cmd1.color.b + d, 0, 255) as u8;
            cmd2.color.b = clamp(cmd2.color.b - d, 0, 255) as u8;
        } else {
            assert!(false, "prop is out of range");
        }
        (cmd1, cmd2)
    }
}

fn fill_path(draw_target: &mut DrawTarget, path: &Path, color: Color, is_antialias: bool) {
    let antialias;
    if is_antialias {
        antialias = AntialiasMode::Gray;
    } else {
        antialias = AntialiasMode::None;
    }
    let draw_options = DrawOptions {
        blend_mode: BlendMode::SrcOver,
        alpha: color.a as f32 / 255.0_f32,
        antialias: antialias,
    };
    let color2 = Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: 0xff,
    };
    draw_target.fill(&path, &Source::Solid(color2), &draw_options);
}

fn fill_ellipse(
    draw_target: &mut DrawTarget,
    cx: u32,
    cy: u32,
    rx: u32,
    ry: u32,
    angle: i32,
    color: Color,
    is_antialias: bool,
) {
    let radius_max = rx.max(ry);
    let cxf = cx as f32;
    let cyf = cy as f32;
    let n = 4 + (radius_max as usize) / 2;

    let mut pb = PathBuilder::new();
    let mut last_x = 0.0_f32;
    let mut last_y = 0.0_f32;
    for i in 0..n {
        let t = (i as f32) * PI2 / (n as f32);
        let (sin_t, cos_t) = t.sin_cos();
        let x = cxf + (rx as f32) * cos_t;
        let y = cyf + (ry as f32) * sin_t;
        if i == 0 {
            pb.move_to(x, y);
            last_x = x;
            last_y = y;
        } else {
            if x != last_x || y != last_y {
                pb.line_to(x, y);
                last_x = x;
                last_y = y;
            }
        }
    }
    pb.close();
    let path = pb.finish();

    let a = Angle::radians((angle as f32) * PI / 180.0);
    let t2 = Transform::create_translation(cxf, cyf);
    let transform: Transform = Transform::create_translation(-cxf, -cyf)
        .post_rotate(a)
        .post_transform(&t2);
    draw_target.set_transform(&transform);

    fill_path(draw_target, &path, color, is_antialias);

    let initial_transform = Transform::identity();
    draw_target.set_transform(&initial_transform);
}

#[inline(always)]
fn rgb2color(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16_u32) + ((g as u32) << 8_u32) + (b as u32)
}

fn diff(rgb2lab: &ColorTable, lab_img: &[Lab], draw_target: &DrawTarget) -> i64 {
    let w = draw_target.width() as u32;
    let h = draw_target.height() as u32;
    let img2: &[u32] = draw_target.get_data();
    let mut sum = 0;
    for y in 0..h {
        for x in 0..w {
            let index = (x + w * y) as usize;
            let color2 = img2[index];
            let lab1 = lab_img[index];
            let lab2 = rgb2lab[(color2 & 0xffffff_u32) as usize];
            let dl = lab1.l - lab2.l;
            let da = lab1.a - lab2.a;
            let db = lab1.b - lab2.b;
            let val = (dl * dl + da * da + db * db) as i64;
            sum += val as i64;
        }
    }
    sum
}

fn avg_color(w: u32, h: u32, img: &[u8]) -> Color {
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
        return Color {
            r: 0,
            g: 0,
            b: 0,
            a: 0xff,
        };
    }
    let r = (sum_r / count) as u8;
    let g = (sum_g / count) as u8;
    let b = (sum_b / count) as u8;

    Color {
        r: r,
        g: g,
        b: b,
        a: 0xff,
    }
}

fn draw_cmd(draw_target: &mut DrawTarget, cmd: &DrawCommand, is_antialias: bool) {
    fill_ellipse(
        draw_target,
        cmd.x,
        cmd.y,
        cmd.rx,
        cmd.ry,
        cmd.angle,
        cmd.color,
        is_antialias,
    );
}

fn copy_img(src: &DrawTarget, dst: &mut DrawTarget) {
    let w = src.width();
    let h = src.height();
    let img_rect = IntRect::from_size(Size2D::new(w as i32, h as i32));
    let zero_point = IntPoint::zero();
    dst.copy_surface(&src, img_rect, zero_point);
}

fn try_draw(
    rgb2lab: &ColorTable,
    draw_target: &DrawTarget,
    tmp_target: &mut DrawTarget,
    lab_img: &[Lab],
    cmd: &DrawCommand,
) -> i64 {
    copy_img(draw_target, tmp_target);
    draw_cmd(tmp_target, &cmd, true);
    let score = diff(&rgb2lab, &lab_img, &tmp_target);
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

fn draw_bg(draw_target: &mut DrawTarget, bg_color_string: &str, w: u32, h: u32, img: &[u8]) {
    println!("bg_color_string:{:?}", &bg_color_string);
    let bg_color;
    if bg_color_string == "avg" {
        bg_color = avg_color(w, h, &img);
    } else {
        let rgb = read_color::rgb(&mut bg_color_string.chars()).unwrap();
        bg_color = Color {
            r: rgb[0],
            g: rgb[1],
            b: rgb[2],
            a: 0xff,
        };
    }
    println!("bg_color:{:?}", &bg_color);
    draw_target.clear(bg_color);
}

fn draw_target_from_vec(w: u32, h: u32, img_data: &Vec<u32>) -> DrawTarget {
    let mut draw_target = DrawTarget::new(w as i32, h as i32);

    let data = draw_target.get_data_mut();
    for y in 0..h {
        for x in 0..w {
            let index = (x + w * y) as usize;
            data[index] = img_data[index];
        }
    }
    draw_target
}

fn vec_from_draw_target(draw_target_mutex: &Mutex<DrawTarget>) -> Vec<u32> {
    let draw_target = draw_target_mutex.lock().unwrap();
    let mut draw_target_data: Vec<u32> = Vec::new();
    let w = draw_target.width();
    let h = draw_target.height();
    let data = draw_target.get_data();
    for y in 0..h {
        for x in 0..w {
            let index = (x + w * y) as usize;
            draw_target_data.push(data[index]);
        }
    }
    draw_target_data
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
    let draw_target_mutex = Mutex::new(DrawTarget::new(w as i32, h as i32));
    {
        let mut draw_target = draw_target_mutex.lock().unwrap();
        draw_bg(&mut draw_target, &bg_color_string, w, h, &img_raw);
        global_best_score = diff(&rgb2lab, &lab_img, &draw_target);
    }

    for t in 0..num {
        let t_ratio = (t as f32) / (num as f32);
        let start = Instant::now();

        let draw_target_data = vec_from_draw_target(&draw_target_mutex);

        let results: Vec<(i64, DrawCommand)> = (0..seed_count)
            .into_par_iter()
            .map(|_| {
                let mut rng = rand::thread_rng();
                let mut tmp_target = DrawTarget::new(w as i32, h as i32);
                let src_target = draw_target_from_vec(w, h, &draw_target_data);

                let mut best_cmd =
                    DrawCommand::rand(w, h, t_ratio, &img_raw, &mut rng, alpha, brush_scale);
                let mut best_score =
                    try_draw(&rgb2lab, &src_target, &mut tmp_target, &lab_img, &best_cmd);

                // optimize
                for _j in 0..optimize_count {
                    let (cmd, cmd2) =
                        DrawCommand::mutate(w, h, t_ratio, &best_cmd, &mut rng, brush_scale);
                    let score;
                    if cmd == best_cmd {
                        score = best_score;
                    } else {
                        score = try_draw(&rgb2lab, &src_target, &mut tmp_target, &lab_img, &cmd);
                    }
                    if score < best_score {
                        best_score = score;
                        best_cmd = cmd;
                    } else if cmd != cmd2 {
                        let score2;
                        if cmd2 == best_cmd {
                            score2 = best_score;
                        } else {
                            score2 =
                                try_draw(&rgb2lab, &src_target, &mut tmp_target, &lab_img, &cmd2);
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
            let mut draw_target = draw_target_mutex.lock().unwrap();
            //draw best cmd
            draw_cmd(&mut draw_target, &best_cmd, true);
        }

        {
            let draw_target = draw_target_mutex.lock().unwrap();
            let img_name = format!("result_{:06}.png", t);
            draw_target.write_png(img_name).unwrap();
        }
    }

    {
        let draw_target = draw_target_mutex.lock().unwrap();
        draw_target.write_png("out.png").unwrap();
    }

    Ok(())
}
/*
todo



check small draw  1x1 2x2 3x2 2x1


paint changed location in alpha white -> score weight
write svg check size
parse outputpath

resize in calc, save in original size

write command
optimize svg
 quantize color, size
 global optimize
  remove unneeded cmd
  mutate cmd

*/
