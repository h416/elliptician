use tiny_skia::*;

use rand::distributions::{Distribution, Uniform};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawCommand {
    pub x: u32,
    pub y: u32,
    pub rx: u32,
    pub ry: u32,
    pub angle: i32, // degree
    pub color: ColorU8,
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
    if res < 0 {
        res += 180;
    }
    if angle > 180 {
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
        let color = ColorU8::from_rgba(0, 0, 0, 0xff);
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
        let color = ColorU8::from_rgba(r, g, b, alpha);
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
            let d = rnd(rng, 1, 8) as i32;
            let red = cmd1.color.red() as i32;
            cmd1.color = ColorU8::from_rgba(
                clamp(red + d, 0, 255) as u8,
                cmd1.color.green(),
                cmd1.color.blue(),
                cmd1.color.alpha(),
            );
            cmd1.color = ColorU8::from_rgba(
                clamp(red - d, 0, 255) as u8,
                cmd1.color.green(),
                cmd1.color.blue(),
                cmd1.color.alpha(),
            );
        } else if prop == 6 {
            let d = rnd(rng, 1, 8);
            let green = cmd1.color.green() as i32;
            cmd1.color = ColorU8::from_rgba(
                cmd1.color.red(),
                clamp(green + d, 0, 255) as u8,
                cmd1.color.blue(),
                cmd1.color.alpha(),
            );
            cmd1.color = ColorU8::from_rgba(
                cmd1.color.red(),
                clamp(green - d, 0, 255) as u8,
                cmd1.color.blue(),
                cmd1.color.alpha(),
            );
        } else if prop == 7 {
            let d = rnd(rng, 1, 8);
            let blue = cmd1.color.blue() as i32;
            cmd1.color = ColorU8::from_rgba(
                cmd1.color.red(),
                cmd1.color.blue(),
                clamp(blue + d, 0, 255) as u8,
                cmd1.color.alpha(),
            );
            cmd1.color = ColorU8::from_rgba(
                cmd1.color.red(),
                cmd1.color.blue(),
                clamp(blue - d, 0, 255) as u8,
                cmd1.color.alpha(),
            );
        } else {
            assert!(false, "prop is out of range");
        }
        (cmd1, cmd2)
    }
}
