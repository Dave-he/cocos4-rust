use crate::math::Vec4;
use super::define::SKY_ILLUM;

pub struct AmbientInfo {
    pub sky_color_hdr: Vec4,
    pub sky_illum_hdr: f32,
    pub ground_albedo_hdr: Vec4,
    pub sky_color_ldr: Vec4,
    pub sky_illum_ldr: f32,
    pub ground_albedo_ldr: Vec4,
}

impl Default for AmbientInfo {
    fn default() -> Self {
        AmbientInfo {
            sky_color_hdr: Vec4::new(0.2, 0.5, 0.8, 1.0),
            sky_illum_hdr: SKY_ILLUM,
            ground_albedo_hdr: Vec4::new(0.2, 0.2, 0.2, 1.0),
            sky_color_ldr: Vec4::new(0.2, 0.5, 0.8, 1.0),
            sky_illum_ldr: SKY_ILLUM,
            ground_albedo_ldr: Vec4::new(0.2, 0.2, 0.2, 1.0),
        }
    }
}

impl AmbientInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_sky_color_hdr(&mut self, val: Vec4) {
        self.sky_color_hdr = val;
    }

    pub fn get_sky_color_hdr(&self) -> &Vec4 {
        &self.sky_color_hdr
    }

    pub fn get_sky_color_ldr(&self) -> &Vec4 {
        &self.sky_color_ldr
    }

    pub fn set_ground_albedo_hdr(&mut self, val: Vec4) {
        self.ground_albedo_hdr = val;
    }

    pub fn get_ground_albedo_hdr(&self) -> &Vec4 {
        &self.ground_albedo_hdr
    }

    pub fn get_ground_albedo_ldr(&self) -> &Vec4 {
        &self.ground_albedo_ldr
    }

    pub fn set_sky_illum_hdr(&mut self, val: f32) {
        self.sky_illum_hdr = val;
    }

    pub fn get_sky_illum_hdr(&self) -> f32 {
        self.sky_illum_hdr
    }

    pub fn get_sky_illum_ldr(&self) -> f32 {
        self.sky_illum_ldr
    }

    pub fn set_sky_illum(&mut self, val: f32) {
        self.sky_illum_hdr = val;
        self.sky_illum_ldr = val;
    }

    pub fn get_sky_illum(&self) -> f32 {
        self.sky_illum_hdr
    }

    pub fn set_sky_color(&mut self, val: Vec4) {
        self.sky_color_hdr = val;
        self.sky_color_ldr = val;
    }

    pub fn set_ground_albedo(&mut self, val: Vec4) {
        self.ground_albedo_hdr = val;
        self.ground_albedo_ldr = val;
    }

    pub fn activate(&mut self, resource: &mut Ambient) {
        resource.sky_color_hdr = self.sky_color_hdr;
        resource.sky_illum_hdr = self.sky_illum_hdr;
        resource.ground_albedo_hdr = self.ground_albedo_hdr;
        resource.sky_color_ldr = self.sky_color_ldr;
        resource.sky_illum_ldr = self.sky_illum_ldr;
        resource.ground_albedo_ldr = self.ground_albedo_ldr;
    }
}

pub struct Ambient {
    pub enabled: bool,
    pub sky_color_hdr: Vec4,
    pub sky_illum_hdr: f32,
    pub ground_albedo_hdr: Vec4,
    pub sky_color_ldr: Vec4,
    pub sky_illum_ldr: f32,
    pub ground_albedo_ldr: Vec4,
    pub mipmap_count: u8,
}

impl Default for Ambient {
    fn default() -> Self {
        Ambient {
            enabled: false,
            sky_color_hdr: Vec4::new(0.2, 0.5, 0.8, 1.0),
            sky_illum_hdr: 0.0,
            ground_albedo_hdr: Vec4::new(0.2, 0.2, 0.2, 1.0),
            sky_color_ldr: Vec4::new(0.2, 0.5, 0.8, 1.0),
            sky_illum_ldr: 0.0,
            ground_albedo_ldr: Vec4::new(0.2, 0.2, 0.2, 1.0),
            mipmap_count: 1,
        }
    }
}

impl Ambient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn initialize(&mut self, info: &AmbientInfo) {
        self.sky_color_hdr = info.sky_color_hdr;
        self.sky_illum_hdr = info.sky_illum_hdr;
        self.ground_albedo_hdr = info.ground_albedo_hdr;
        self.sky_color_ldr = info.sky_color_ldr;
        self.sky_illum_ldr = info.sky_illum_ldr;
        self.ground_albedo_ldr = info.ground_albedo_ldr;
    }

    pub fn set_enabled(&mut self, val: bool) {
        self.enabled = val;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_sky_color(&self) -> &Vec4 {
        &self.sky_color_hdr
    }

    pub fn set_sky_color(&mut self, color: Vec4) {
        self.sky_color_hdr = color;
    }

    pub fn get_sky_illum(&self) -> f32 {
        self.sky_illum_hdr
    }

    pub fn set_sky_illum(&mut self, illum: f32) {
        self.sky_illum_hdr = illum;
    }

    pub fn get_ground_albedo(&self) -> &Vec4 {
        &self.ground_albedo_hdr
    }

    pub fn set_ground_albedo(&mut self, color: Vec4) {
        self.ground_albedo_hdr = color;
    }

    pub fn get_mipmap_count(&self) -> u8 {
        self.mipmap_count
    }

    pub fn set_mipmap_count(&mut self, count: u8) {
        self.mipmap_count = count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ambient_new() {
        let ambient = Ambient::new();
        assert!(!ambient.enabled);
        assert_eq!(ambient.mipmap_count, 1);
    }

    #[test]
    fn test_ambient_info_activate() {
        let mut info = AmbientInfo::new();
        info.set_sky_illum(50000.0);
        let mut ambient = Ambient::new();
        info.activate(&mut ambient);
        assert_eq!(ambient.sky_illum_hdr, 50000.0);
    }

    #[test]
    fn test_ambient_set_enabled() {
        let mut ambient = Ambient::new();
        ambient.set_enabled(true);
        assert!(ambient.is_enabled());
    }
}
