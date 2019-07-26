use std::time::Instant;

use raqote::*;

use euclid::{Angle, Size2D};

use rand::distributions::{Distribution, Uniform};

type Color = SolidSource;

const PI: f32 = std::f32::consts::PI;
const PI2: f32 = 2.0_f32 * PI;

const ANGLE_MIN: f32 = -PI / 4.0;
const ANGLE_MAX: f32 = PI / 4.0;

#[derive(Clone, Debug)]
pub struct DrawCommand {
    x: f32,
    y: f32,
    rx: f32,
    ry: f32,
    angle: f32,
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

impl DrawCommand {
    pub fn rand(
        w: u32,
        h: u32,
        xx: u32,
        yy: u32,
        img: &[u8],
        rng: &mut rand::rngs::ThreadRng,
        alpha: u8,
    ) -> DrawCommand {
        let w1 = w - 1;
        let h1 = h - 1;
        //let ux: u32 = rnd(rng, 0, w1) as u32;
        //let uy: u32 = rnd(rng, 0, h1) as u32;
        let ux = clamp(xx as i32 + rnd(rng, -16, 16), 0, w1 as i32) as u32;
        let uy = clamp(yy as i32 + rnd(rng, -16, 16), 0, h1 as i32) as u32;
        let x = ux as f32;
        let y = uy as f32;

        let index = 4 * (ux + w * uy) as usize;
        let r: u8 = img[index];
        let g: u8 = img[index + 1];
        let b: u8 = img[index + 2];

        let r_max = (w1 + h1) / 16;
        let rx: f32 = rnd(rng, 1, r_max) as f32;
        let ry: f32 = rnd(rng, 1, r_max) as f32;
        let angle: f32 = rnd(rng, ANGLE_MIN, ANGLE_MAX);
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
        width: u32,
        height: u32,
        cmd: &DrawCommand,
        rng: &mut rand::rngs::ThreadRng,
    ) -> DrawCommand {
        let w1: f32 = (width - 1) as f32;
        let h1: f32 = (height - 1) as f32;
        let mut x = cmd.x;
        let mut y = cmd.y;
        let mut rx = cmd.rx;
        let mut ry = cmd.ry;
        let mut angle = cmd.angle;
        let mut r = cmd.color.r;
        let mut g = cmd.color.g;
        let mut b = cmd.color.b;
        let a = cmd.color.a;

        let prop = rnd(rng, 0, 3) as u8;
        if prop == 0 {
            x = clamp(x + rnd(rng, -16.0, 16.0), 0.0, w1);
            y = clamp(y + rnd(rng, -16.0, 16.0), 0.0, h1);
        } else if prop == 1 {
            rx = clamp(rx + rnd(rng, -16.0, 16.0), 0.0, w1);
            ry = clamp(ry + rnd(rng, -16.0, 16.0), 0.0, h1);
        } else if prop == 2 {
            angle += rnd(rng, -PI / 180.0 * 4.0, PI / 180.0 * 4.0);
            // angle = angle.min(ANGLE_MAX).max(ANGLE_MIN);
            if angle < -PI {
                angle += 2.0 * PI;
            }
            if angle > PI {
                angle -= 2.0 * PI;
            }
        } else if prop == 3 {
            r = clamp(r as i32 + rnd(rng, -16, 16), 0, 255) as u8;
            g = clamp(g as i32 + rnd(rng, -16, 16), 0, 255) as u8;
            b = clamp(b as i32 + rnd(rng, -16, 16), 0, 255) as u8;
        }
        let color = Color {
            r: r,
            g: g,
            b: b,
            a: a,
        };
        let res = DrawCommand {
            x,
            y,
            rx,
            ry,
            angle,
            color,
        };
        res
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
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
    angle: f32,
    color: Color,
    is_antialias: bool,
) {
    let radius_max = rx.max(ry);
    if radius_max <= 0.0_f32 {
        return;
    }

    let n = 4 + (radius_max.ceil() as usize) / 2;

    let mut pb = PathBuilder::new();
    let mut last_x = 0.0_f32;
    let mut last_y = 0.0_f32;
    for i in 0..n {
        let t = (i as f32) * PI2 / (n as f32);
        let (sin_t, cos_t) = t.sin_cos();
        let x = cx + rx * cos_t;
        let y = cy + ry * sin_t;
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

    let a = Angle::radians(angle);
    let t2 = Transform::create_translation(cx, cy);
    let transform: Transform = Transform::create_translation(-cx, -cy)
        .post_rotate(a)
        .post_transform(&t2);
    draw_target.set_transform(&transform);

    fill_path(draw_target, &path, color, is_antialias);

    let initial_transform = Transform::identity();
    draw_target.set_transform(&initial_transform);
}

fn diff(img: &[u8], draw_target: &DrawTarget) -> (i64, u32, u32) {
    let w = draw_target.width() as u32;
    let h = draw_target.height() as u32;
    let img2: &[u32] = draw_target.get_data();
    let mut sum = 0;
    let mut max_diff = 0;
    let mut diff_x = 0;
    let mut diff_y = 0;
    for y in 0..h {
        for x in 0..w {
            let index2 = (x + w * y) as usize;
            let index = 4 * index2;
            let r1 = img[index] as i32;
            let g1 = img[index + 1] as i32;
            let b1 = img[index + 2] as i32;
            let color2 = &img2[index2];

            let r2 = ((color2 >> 16) & 0xffu32) as i32;
            let g2 = ((color2 >> 8) & 0xffu32) as i32;
            let b2 = ((color2 >> 0) & 0xffu32) as i32;

            let dr = r1 - r2;
            let dg = g1 - g2;
            let db = b1 - b2;
            let val = dr * dr + dg * dg + db * db;
            if val > max_diff {
                max_diff = val;
                diff_x = x;
                diff_y = y;
            }
            sum += val as i64;
        }
    }
    (sum, diff_x, diff_y)
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
    draw_target: &DrawTarget,
    tmp_target: &mut DrawTarget,
    img: &[u8],
    cmd: &DrawCommand,
) -> i64 {
    copy_img(draw_target, tmp_target);
    draw_cmd(tmp_target, &cmd, false);
    let (score, _diff_x, _diff_y) = diff(&img, &tmp_target);
    score
}
fn main() -> Result<(), Box<std::error::Error>> {
    let img = image::open("examples/monalisa_s.jpg").unwrap().to_rgba();
    let w = img.width();
    let h = img.height();
    println!("{}x{}", w, h);
    let img_raw = img.into_raw();
    let bg_color = avg_color(w, h, &img_raw);

    let mut draw_target = DrawTarget::new(w as i32, h as i32);

    draw_target.clear(bg_color);

    let mut rng = rand::thread_rng();

    let mut tmp_target = DrawTarget::new(w as i32, h as i32);
    let alpha = 128;

    let mut best_score = 0;
    let mut best_cmd = DrawCommand::rand(w, h, 0, 0, &img_raw, &mut rng, alpha);

    for t in 0..1000 {
        let start = Instant::now();
        let (_score, diff_x, diff_y) = diff(&img_raw, &draw_target);

        for i in 0..32 {
            let cmd = DrawCommand::rand(w, h, diff_x, diff_y, &img_raw, &mut rng, alpha);
            let score = try_draw(&draw_target, &mut tmp_target, &img_raw, &cmd);
            if i == 0 || score < best_score {
                best_score = score;
                best_cmd = cmd;
            }

            // optimize
            for _j in 0..64 {
                let cmd = DrawCommand::mutate(w, h, &best_cmd, &mut rng);
                let score = try_draw(&draw_target, &mut tmp_target, &img_raw, &cmd);
                if score < best_score {
                    best_score = score;
                    best_cmd = cmd;
                }
            }
        }
        let duration = start.elapsed();
        println!("{} : {} {:?}", t, best_score, duration);

        //draw best cmd
        draw_cmd(&mut draw_target, &best_cmd, true);

        let img_name = format!("result_{:06}.png", t);
        draw_target.write_png(img_name).unwrap();
    }

    draw_target.write_png("out.png").unwrap();

    Ok(())
}
/*
todo


angle -> rad to degree
cmd member user int?  i32?

check small draw  1x1 2x2 3x2 2x1

profiling

diff use different color space

parse argument

resize in calc, save in original size
write svg check size
write command
optimize svg
 quantize color, angle coord
 global optimize
  remove unneeded cmd
  mutate cmd

*/
