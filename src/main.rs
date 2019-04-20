use footile::{FillRule, PathBuilder, Plotter, Raster, Rgba8, Transform};
const PI: f32 = std::f32::consts::PI;

fn fill_rect(
    p: &mut Plotter,
    r: &mut Raster<Rgba8>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: Rgba8,
) {
    let rect = PathBuilder::new()
        .absolute()
        .move_to(x, y)
        .line_to(x + width, y)
        .line_to(x + width, y + height)
        .line_to(x, y + height)
        .line_to(x, y)
        .close()
        .build();
    let mask = p.fill(&rect, FillRule::NonZero);
    let pixels = r.as_slice_mut();
    let mask_pixels = mask.pixels();
    dst_over(pixels, mask_pixels, color);
    p.clear_mask();
}

fn fill_circle(p: &mut Plotter, r: &mut Raster<Rgba8>, x: f32, y: f32, radius: f32, color: Rgba8) {
    fill_elipse(p, r, x, y, radius, radius, color);
}

fn fill_elipse(
    p: &mut Plotter,
    r: &mut Raster<Rgba8>,
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
    color: Rgba8,
) {
    let radius_max = rx.max(ry);
    let n = 4 * (radius_max.ceil() as usize);
    if n == 0 {
        return;
    }
    let mut builder = PathBuilder::new().absolute();
    for i in 0..n {
        let t = (i as f32) * 2.0f32 * PI / (n as f32);
        let cos_t = t.cos();
        let sin_t = t.sin();
        let x = cx + rx * cos_t;
        let y = cy + ry * sin_t;
        if i == 0 {
            builder = builder.move_to(x, y);
        } else {
            builder = builder.line_to(x, y);
        }
    }

    let transform = Transform::new_translate(-cx, -cy)
        .rotate(PI * 0.25)
        .translate(cx, cy);
    p.set_transform(transform);

    let shape = builder.close().build();
    let mask = p.fill(&shape, FillRule::NonZero);
    let pixels = r.as_slice_mut();
    let mask_pixels = mask.pixels();
    dst_over(pixels, mask_pixels, color);
    p.clear_mask();

    p.set_transform(Transform::new());
}

fn draw_line(
    p: &mut Plotter,
    r: &mut Raster<Rgba8>,
    x: f32,
    y: f32,
    x2: f32,
    y2: f32,
    color: Rgba8,
) {
    let rect = PathBuilder::new()
        .absolute()
        .move_to(x, y)
        .line_to(x2, y2)
        .close()
        .build();
    let mask = p.stroke(&rect);
    let pixels = r.as_slice_mut();
    let mask_pixels = mask.pixels();

    dst_over(pixels, mask_pixels, color);

    p.clear_mask();
}
fn diff(w: u32, h: u32, img: &[u8], img2: &[Rgba8]) -> i64 {
    let mut sum = 0;
    for y in 0..h {
        for x in 0..w {
            let index2 = (x + w * y) as usize;
            let index = 4 * index2;
            let r1 = img[index] as i64;
            let g1 = img[index + 1] as i64;
            let b1 = img[index + 2] as i64;
            let a1 = img[index + 3] as i64;
            let color2 = &img2[index2];
            let r2 = color2.red() as i64;
            let g2 = color2.green() as i64;
            let b2 = color2.blue() as i64;
            let a2 = color2.alpha() as i64;
            /*
            if y < 2 {
                println!("{} {} {} {}", r1, g1, b1, a1);
                println!(" {} {} {} {}", r2, g2, b2, a2);
            }
            */
            let dr = r1 - r2;
            let dg = g1 - g2;
            let db = b1 - b2;
            let a = a1 * a2;
            let val = (dr * dr + dg * dg + db * db) * a;
            sum += val;
        }
    }
    sum
}

fn avg_color(w: u32, h: u32, img: &[u8]) -> Rgba8 {
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
    let r = (sum_r / count) as u8;
    let g = (sum_g / count) as u8;
    let b = (sum_b / count) as u8;

    Rgba8::rgb(r, g, b)
}
fn main() -> Result<(), Box<std::error::Error>> {
    let img = image::open("examples/monalisa.jpg").unwrap().to_rgba();

    println!("dimensions {:?}", img.dimensions());
    let w = img.width();
    let h = img.height();

    let img_raw = img.into_raw();

    let avg_color = avg_color(w, h, &img_raw);

    /*let rect = PathBuilder::new()
    .absolute()
    .move_to(10.0, 10.0)
    .line_to(310.0, 10.0)
    .line_to(310.0, 230.0)
    .line_to(10.0, 230.0)
    .close()
    .build();*/
    /*let rect2 = PathBuilder::new()
        .absolute()
        .move_to(0.0, 0.0)
        .line_to(80.0, 0.0)
        .line_to(80.0, 80.0)
        .line_to(0.0, 80.0)
        .close()
        .build();
    */
    /*

    */
    let mut p = Plotter::new(w, h);
    let mut r: Raster<Rgba8> = Raster::new(p.width(), p.height());
    //r.over(p.fill(&rect, FillRule::NonZero), Rgba8::new(0, 0, 255, 255));
    /*
    r.over(
        p.fill(&rect2, FillRule::NonZero),
        Rgba8::new(255, 0, 0, 255),
    );
    */

    fill_rect(&mut p, &mut r, 0.0, 0.0, w as f32, h as f32, avg_color);

    fill_rect(
        &mut p,
        &mut r,
        0.0,
        0.0,
        320.0,
        240.0,
        Rgba8::new(10, 10, 10, 255),
    );

    fill_rect(
        &mut p,
        &mut r,
        20.0,
        10.0,
        280.0,
        220.0,
        Rgba8::new(0, 0, 255, 255),
    );

    fill_rect(
        &mut p,
        &mut r,
        0.0,
        0.0,
        80.0,
        80.0,
        Rgba8::new(255, 0, 0, 230),
    );

    fill_rect(
        &mut p,
        &mut r,
        50.0,
        50.0,
        150.0,
        400.0,
        Rgba8::new(255, 0, 255, 230),
    );

    fill_circle(
        &mut p,
        &mut r,
        50.0,
        50.0,
        50.0,
        Rgba8::new(255, 0, 255, 100),
    );
    fill_elipse(
        &mut p,
        &mut r,
        160.0,
        120.0,
        100.0,
        80.0,
        Rgba8::new(0, 255, 255, 100),
    );

    draw_line(
        &mut p,
        &mut r,
        10.0,
        10.0,
        20.0,
        10.0,
        Rgba8::new(255, 255, 255, 255),
    );

    let fish = PathBuilder::new()
        .relative()
        .pen_width(3.0)
        .move_to(112.0, 24.0)
        .line_to(-32.0, 24.0)
        .cubic_to(-96.0, -48.0, -96.0, 80.0, 0.0, 32.0)
        .line_to(32.0, 24.0)
        .line_to(-16.0, -40.0)
        .close()
        .build();

    let color = Rgba8::new(0, 255, 0, 150);
    let mask = p.fill(&fish, FillRule::NonZero);
    let pixels = r.as_slice_mut();
    let mask_pixels = mask.pixels();
    dst_over(pixels, mask_pixels, color);

    //let color = Rgba8::new(0, 255, 0, 128);
    //r.over(p.stroke(&fish), color);
    /*let mask = p.stroke(&fish);
    let pixels = r.as_slice_mut();
    let mask_pixels = mask.pixels();
    over_fallback(pixels, mask_pixels, color);
    */

    //let rastered_img = r.as_u8_slice();

    let score = diff(w, h, &img_raw, pixels);
    println!("score:{}", score);

    r.write_png("result.png")?;

    /*
    TODO

    struct graphics

    draw_rect
    draw_circle
    draw_elipse

    draw rotated rect
    draw rotated elipse


    */

    Ok(())
}

fn dst_over_alpha(src: Rgba8, dst: Rgba8, mask: u8) -> Rgba8 {
    // https://en.wikipedia.org/wiki/Alpha_compositing
    // http://www.svgopen.org/2005/papers/abstractsvgopen/index.html#porterduff
    // dst-over
    let r1 = src.red() as i32;
    let g1 = src.green() as i32;
    let b1 = src.blue() as i32;
    let a1 = src.alpha() as i32;
    let r2 = dst.red() as i32;
    let g2 = dst.green() as i32;
    let b2 = dst.blue() as i32;
    let a2 = (dst.alpha() as i32) * (mask as i32) / 255;
    let inv_a2 = 255 - a2;
    let a = a1 * inv_a2 + a2 * 255;
    if a == 0 {
        return src;
    }
    let a11 = a1 * inv_a2;
    let a22 = a2 * 255;
    let r = (r1 * a11 + r2 * a22) / a;
    let g = (g1 * a11 + g2 * a22) / a;
    let b = (b1 * a11 + b2 * a22) / a;
    Rgba8::new(r as u8, g as u8, b as u8, (a / 255) as u8)
}
fn dst_over(pix: &mut [Rgba8], mask: &[u8], clr: Rgba8) {
    for (p, m) in pix.iter_mut().zip(mask) {
        //let out = over_alpha(clr, *p, *m);
        let out = dst_over_alpha(*p, clr, *m);
        *p = out;
    }
}
