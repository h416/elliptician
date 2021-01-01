use tiny_skia::*;

const PI: f32 = std::f32::consts::PI;

pub fn fill_ellipse(
    canvas: &mut Canvas,
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
    angle_degree: f32,
    color: &ColorU8,
    is_antialias: bool,
) {
    if rx == 0.0 || ry == 0.0 {
        return;
    }

    let mut r = rx;
    let mut sx = 1.0;
    let mut sy = ry / rx;
    if rx < ry {
        r = ry;
        sx = rx / ry;
        sy = 1.0;
    }

    let path = PathBuilder::from_circle(0.0, 0.0, r).unwrap();

    let angle_radian = angle_degree * PI / 180.0;
    let sc = angle_radian.sin_cos();
    let sin = sc.0;
    let cos = sc.1;
    let rotate = Transform::from_row(cos, sin, -sin, cos, 0.0, 0.0).unwrap();

    let scale = Transform::from_scale(sx, sy).unwrap();
    let t = Transform::from_translate(cx, cy).unwrap();
    let mut transform = scale.post_concat(&rotate).unwrap();
    transform = transform.post_concat(&t).unwrap();

    canvas.set_transform(transform);

    let mut paint = Paint::default();
    paint.set_color_rgba8(color.red(), color.green(), color.blue(), color.alpha());
    paint.anti_alias = is_antialias;

    canvas.fill_path(&path, &paint, FillRule::Winding);

    canvas.reset_transform();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fill_ellipse_test() {
        assert_eq!(1, 1);

        let mut pixmap = Pixmap::new(16, 16).unwrap();
        let mut canvas = Canvas::from(pixmap.as_mut());
        fill_ellipse(
            &mut canvas,
            8.0,
            8.0,
            8.0,
            8.0,
            0.0,
            &ColorU8::from_rgba(255, 200, 100, 255),
            true,
        );

        let color_result = pixmap.pixel(8, 8);
        assert_eq!(color_result.is_some(), true);
        let color = color_result.unwrap();
        assert_eq!(color.red(), 255);
        assert_eq!(color.green(), 200);
        assert_eq!(color.blue(), 100);
    }
}
