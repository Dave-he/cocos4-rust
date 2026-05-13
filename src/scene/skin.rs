pub struct SkinInfo {
    pub enabled: bool,
    pub blur_radius: f32,
    pub sss_intensity: f32,
}

impl Default for SkinInfo {
    fn default() -> Self {
        SkinInfo {
            enabled: true,
            blur_radius: 0.01,
            sss_intensity: 3.0,
        }
    }
}

impl SkinInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_enabled(&mut self, val: bool) {
        self.enabled = val;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_blur_radius(&mut self, val: f32) {
        self.blur_radius = val;
    }

    pub fn get_blur_radius(&self) -> f32 {
        self.blur_radius
    }

    pub fn set_sss_intensity(&mut self, val: f32) {
        self.sss_intensity = val;
    }

    pub fn get_sss_intensity(&self) -> f32 {
        self.sss_intensity
    }

    pub fn activate(&mut self, resource: &mut Skin) {
        resource.enabled = self.enabled;
        resource.blur_radius = self.blur_radius;
        resource.sss_intensity = self.sss_intensity;
    }
}

pub struct Skin {
    pub enabled: bool,
    pub blur_radius: f32,
    pub sss_intensity: f32,
}

impl Default for Skin {
    fn default() -> Self {
        Skin {
            enabled: true,
            blur_radius: 0.01,
            sss_intensity: 3.0,
        }
    }
}

impl Skin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn initialize(&mut self, info: &SkinInfo) {
        self.enabled = info.enabled;
        self.blur_radius = info.blur_radius;
        self.sss_intensity = info.sss_intensity;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, val: bool) {
        self.enabled = val;
    }

    pub fn get_blur_radius(&self) -> f32 {
        self.blur_radius
    }

    pub fn set_blur_radius(&mut self, val: f32) {
        self.blur_radius = val;
    }

    pub fn get_sss_intensity(&self) -> f32 {
        self.sss_intensity
    }

    pub fn set_sss_intensity(&mut self, val: f32) {
        self.sss_intensity = val;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skin_new() {
        let skin = Skin::new();
        assert!(skin.enabled);
        assert_eq!(skin.blur_radius, 0.01);
        assert_eq!(skin.sss_intensity, 3.0);
    }

    #[test]
    fn test_skin_info_activate() {
        let mut info = SkinInfo::new();
        info.set_blur_radius(0.05);
        let mut skin = Skin::new();
        info.activate(&mut skin);
        assert_eq!(skin.blur_radius, 0.05);
    }
}
