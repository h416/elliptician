use lab::Lab;

pub struct ColorConverter {
    table: Vec<Lab>,
}

impl ColorConverter {
    pub fn new() -> ColorConverter {
        let mut table = Vec::new();
        let lab0 = Lab::from_rgb(&[0, 0, 0]);
        table.resize(16777216, lab0);
        for r in 0..=255 {
            for g in 0..=255 {
                for b in 0..=255 {
                    let color = ((r as u32) << 16_u32) + ((g as u32) << 8_u32) + (b as u32);
                    let lab = Lab::from_rgb(&[r, g, b]);
                    table[color as usize] = lab;
                }
            }
        }

        ColorConverter { table: table }
    }

    pub fn get_lab(&self, r: u8, g: u8, b: u8) -> Lab {
        let value = ((r as u32) << 16_u32) + ((g as u32) << 8_u32) + (b as u32);
        self.table[value as usize]
    }

    pub fn lab_image(&self, w: u32, h: u32, img: &[u8]) -> Vec<Lab> {
        let mut result = Vec::new();

        for y in 0..h {
            for x in 0..w {
                let index2 = (x + w * y) as usize;
                let index = 4 * index2;
                let r = img[index];
                let g = img[index + 1];
                let b = img[index + 2];
                let lab = self.get_lab(r, g, b);
                result.push(lab);
            }
        }

        result
    }
}
