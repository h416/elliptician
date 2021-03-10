use tiny_skia::*;

use lab::Lab;

use crate::ColorConverter;

pub fn diff(
    color_converter: &ColorConverter,
    lab_img: &[Lab],
    pixmap: &mut Pixmap,
    mse_ratio: f32,
) -> f32 {
    let w = pixmap.width();
    let h = pixmap.height();
    let img2: &[PremultipliedColorU8] = pixmap.pixels_mut();

    let mut sum = 0.0_f32;
    for y in 0..h {
        for x in 0..w {
            let index = (x + w * y) as usize;
            let color2 = img2[index];
            let lab1 = lab_img[index];
            assert_eq!(color2.is_opaque(), true);
            let lab2 = color_converter.get_lab(color2.red(), color2.green(), color2.blue());

            let dl = (lab1.l - lab2.l) * 0.01_f32;
            let da = (lab1.a - lab2.a) * 0.01_f32;
            let db = (lab1.b - lab2.b) * 0.01_f32;
            let val = (dl * dl + da * da + db * db) / 3.0_f32;
            sum += val;
        }
    }
    let mse = sum / ((w * h) as f32);

    // https://en.wikipedia.org/wiki/Structural_similarity

    let block_size = 8;
    let inv_samples = 1.0_f32 / (block_size * block_size) as f32;
    let x_block_num = w / block_size;
    let y_block_num = h / block_size;

    if x_block_num == 0 || y_block_num == 0 {
        return 0.0;
    }

    let mut ssims: Vec<f32> = Vec::new();
    for by in 0..y_block_num {
        for bx in 0..x_block_num {
            let base_offset = bx * block_size + by * block_size * w;
            let mut sum_l1 = 0.0_f32;
            let mut sum_l2 = 0.0_f32;
            let mut sum_l11 = 0.0_f32;
            let mut sum_l22 = 0.0_f32;
            let mut sum_l12 = 0.0_f32;

            let mut sum_a1 = 0.0_f32;
            let mut sum_a2 = 0.0_f32;
            let mut sum_a11 = 0.0_f32;
            let mut sum_a22 = 0.0_f32;
            let mut sum_a12 = 0.0_f32;

            let mut sum_b1 = 0.0_f32;
            let mut sum_b2 = 0.0_f32;
            let mut sum_b11 = 0.0_f32;
            let mut sum_b22 = 0.0_f32;
            let mut sum_b12 = 0.0_f32;

            for j in 0..block_size {
                for i in 0..block_size {
                    let index = (base_offset + i + w * j) as usize;
                    let color2 = img2[index];
                    let lab1 = lab_img[index];
                    assert_eq!(color2.is_opaque(), true);
                    let lab2 = color_converter.get_lab(color2.red(), color2.green(), color2.blue());
                    let l1 = lab1.l;
                    let a1 = lab1.a;
                    let b1 = lab1.b;
                    let l2 = lab2.l;
                    let a2 = lab2.a;
                    let b2 = lab2.b;

                    sum_l1 += l1;
                    sum_l2 += l2;
                    sum_l11 += l1 * l1;
                    sum_l22 += l2 * l2;
                    sum_l12 += l1 * l2;

                    sum_a1 += a1;
                    sum_a2 += a2;
                    sum_a11 += a1 * a1;
                    sum_a22 += a2 * a2;
                    sum_a12 += a1 * a2;

                    sum_b1 += b1;
                    sum_b2 += b2;
                    sum_b11 += b1 * b1;
                    sum_b22 += b2 * b2;
                    sum_b12 += b1 * b2;
                }
            }

            {
                let avg_l1 = sum_l1 * inv_samples;
                let var_l1 = sum_l11 * inv_samples - (avg_l1 * avg_l1);
                let avg_l2 = sum_l2 * inv_samples;
                let var_l2 = sum_l22 * inv_samples - (avg_l2 * avg_l2);
                let cov_l = sum_l12 * inv_samples - (avg_l1 * avg_l2);
                let ssim_l = get_ssim(avg_l1, avg_l2, var_l1, var_l2, cov_l);
                ssims.push(ssim_l);
            }

            {
                let avg_a1 = sum_a1 * inv_samples;
                let var_a1 = sum_a11 * inv_samples - (avg_a1 * avg_a1);
                let avg_a2 = sum_a2 * inv_samples;
                let var_a2 = sum_a22 * inv_samples - (avg_a2 * avg_a2);
                let cov_a = sum_a12 * inv_samples - (avg_a1 * avg_a2);
                let ssim_a = get_ssim(avg_a1, avg_a2, var_a1, var_a2, cov_a);
                ssims.push(ssim_a);
            }

            {
                let avg_b1 = sum_b1 * inv_samples;
                let var_b1 = sum_b11 * inv_samples - (avg_b1 * avg_b1);
                let avg_b2 = sum_b2 * inv_samples;
                let var_b2 = sum_b22 * inv_samples - (avg_b2 * avg_b2);
                let cov_b = sum_b12 * inv_samples - (avg_b1 * avg_b2);
                let ssim_b = get_ssim(avg_b1, avg_b2, var_b1, var_b2, cov_b);
                ssims.push(ssim_b);
            }
        }
    }

    let sum: f32 = Iterator::sum(ssims.iter());
    let ssim = sum / (ssims.len() as f32);
    let dssim = (1.0_f32 - ssim) * 0.5_f32;
    let ratio = mse_ratio.min(1.0).max(0.0);
    let result = ratio * mse + (1.0_f32 - ratio) * dssim;
    // println!("{} {} {} {}", result, mse, dssim, ratio);
    result
}

fn get_ssim(avg1: f32, avg2: f32, var1: f32, var2: f32, cov: f32) -> f32 {
    let c1 = 6.5025_f32; // (0.01*255.0)^2
    let c2 = 58.5225_f32; // (0.03*255)^2
    let ssim_num = (2.0_f32 * avg1 * avg2 + c1) * (2.0_f32 * cov + c2);
    let ssim_den = (avg1 * avg1 + avg2 * avg2 + c1) * (var1 + var2 + c2);
    let ssim = ssim_num / ssim_den;
    ssim
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dssim_test() {
        const WIDTH: u32 = 32;
        const HEIGHT: u32 = 32;

        let mut pixmap = Pixmap::new(WIDTH, HEIGHT).unwrap();
        pixmap.fill(Color::from_rgba8(0, 0, 0, 255));

        let img_raw = vec![0_u8; (WIDTH * HEIGHT * 4) as usize];

        let color_converter = ColorConverter::new();
        let lab_img = color_converter.lab_image(WIDTH, HEIGHT, &img_raw);

        let res = diff(&color_converter, &lab_img, &mut pixmap, 0.1);

        assert_eq!(res, 0.0_f32);
    }
}
