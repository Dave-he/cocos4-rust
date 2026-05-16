#[derive(Debug, Clone)]
pub struct HeightField {
    pub width: u32,
    pub height: u32,
    pub min_height: f32,
    pub max_height: f32,
    pub data: Vec<f32>,
}

impl HeightField {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            min_height: 0.0,
            max_height: 0.0,
            data: vec![0.0; (width * height) as usize],
        }
    }

    pub fn set_height(&mut self, x: u32, z: u32, value: f32) {
        if x < self.width && z < self.height {
            let idx = (z * self.width + x) as usize;
            self.data[idx] = value;
        }
    }

    pub fn get_height(&self, x: u32, z: u32) -> f32 {
        if x < self.width && z < self.height {
            self.data[(z * self.width + x) as usize]
        } else {
            0.0
        }
    }

    pub fn sample_bilinear(&self, x: f32, z: f32) -> f32 {
        let fx = x.floor();
        let fz = z.floor();
        let cx = x - fx;
        let cz = z - fz;

        let x0 = fx.clamp(0.0, (self.width - 1) as f32) as u32;
        let x1 = (fx + 1.0).clamp(0.0, (self.width - 1) as f32) as u32;
        let z0 = fz.clamp(0.0, (self.height - 1) as f32) as u32;
        let z1 = (fz + 1.0).clamp(0.0, (self.height - 1) as f32) as u32;

        let h00 = self.get_height(x0, z0);
        let h10 = self.get_height(x1, z0);
        let h01 = self.get_height(x0, z1);
        let h11 = self.get_height(x1, z1);

        let h0 = h00 * (1.0 - cx) + h10 * cx;
        let h1 = h01 * (1.0 - cx) + h11 * cx;

        h0 * (1.0 - cz) + h1 * cz
    }

    pub fn update_min_max(&mut self) {
        self.min_height = self.data.iter().cloned().fold(f32::MAX, f32::min);
        self.max_height = self.data.iter().cloned().fold(f32::MIN, f32::max);
    }

    pub fn get_size_in_vertices(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn get_total_vertices(&self) -> usize {
        (self.width * self.height) as usize
    }

    pub fn clear(&mut self) {
        self.data.fill(0.0);
        self.min_height = 0.0;
        self.max_height = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_height_field_new() {
        let hf = HeightField::new(64, 64);
        assert_eq!(hf.width, 64);
        assert_eq!(hf.get_total_vertices(), 64 * 64);
    }

    #[test]
    fn test_set_get_height() {
        let mut hf = HeightField::new(32, 32);
        hf.set_height(10, 15, 100.0);
        assert_eq!(hf.get_height(10, 15), 100.0);
    }

    #[test]
    fn test_bilinear_sampling() {
        let mut hf = HeightField::new(4, 4);
        hf.set_height(0, 0, 0.0);
        hf.set_height(1, 0, 10.0);
        hf.set_height(0, 1, 10.0);
        hf.set_height(1, 1, 20.0);
        let mid = hf.sample_bilinear(0.5, 0.5);
        assert!((mid - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_min_max() {
        let mut hf = HeightField::new(4, 4);
        hf.set_height(0, 0, -5.0);
        hf.set_height(2, 2, 100.0);
        hf.update_min_max();
        assert_eq!(hf.min_height, -5.0);
        assert_eq!(hf.max_height, 100.0);
    }

    #[test]
    fn test_out_of_bounds() {
        let hf = HeightField::new(8, 8);
        assert_eq!(hf.get_height(100, 100), 0.0);
    }
}
