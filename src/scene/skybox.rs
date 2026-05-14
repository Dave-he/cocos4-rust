use crate::math::Mat4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnvironmentLightingType {
    #[default]
    HemisphereDiffuse = 0,
    AutogenHemisphereDiffuseWithReflection = 1,
    DiffuseMapWithReflection = 2,
}

pub struct SkyboxInfo {
    pub enabled: bool,
    pub use_hdr: bool,
    pub use_ibl: bool,
    pub apply_diffuse_map: bool,
    pub env_lighting_type: EnvironmentLightingType,
    pub rotation_angle: f32,
}

impl Default for SkyboxInfo {
    fn default() -> Self {
        SkyboxInfo {
            enabled: false,
            use_hdr: true,
            use_ibl: false,
            apply_diffuse_map: false,
            env_lighting_type: EnvironmentLightingType::HemisphereDiffuse,
            rotation_angle: 0.0,
        }
    }
}

impl SkyboxInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_enabled(&mut self, val: bool) {
        self.enabled = val;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_use_ibl(&mut self, val: bool) {
        self.use_ibl = val;
    }

    pub fn is_use_ibl(&self) -> bool {
        self.use_ibl
    }

    pub fn set_use_hdr(&mut self, val: bool) {
        self.use_hdr = val;
    }

    pub fn is_use_hdr(&self) -> bool {
        self.use_hdr
    }

    pub fn set_apply_diffuse_map(&mut self, val: bool) {
        self.apply_diffuse_map = val;
    }

    pub fn is_apply_diffuse_map(&self) -> bool {
        self.apply_diffuse_map
    }

    pub fn set_env_lighting_type(&mut self, val: EnvironmentLightingType) {
        self.env_lighting_type = val;
    }

    pub fn get_env_lighting_type(&self) -> EnvironmentLightingType {
        self.env_lighting_type
    }

    pub fn set_rotation_angle(&mut self, val: f32) {
        self.rotation_angle = val;
    }

    pub fn get_rotation_angle(&self) -> f32 {
        self.rotation_angle
    }

    pub fn activate(&mut self, resource: &mut Skybox) {
        resource.enabled = self.enabled;
        resource.use_hdr = self.use_hdr;
        resource.use_ibl = self.use_ibl;
        resource.use_diffuse_map = self.apply_diffuse_map;
        resource.env_lighting_type = self.env_lighting_type;
        resource.rotation_angle = self.rotation_angle;
    }
}

pub struct Skybox {
    pub enabled: bool,
    pub use_ibl: bool,
    pub use_hdr: bool,
    pub use_diffuse_map: bool,
    pub activated: bool,
    pub env_lighting_type: EnvironmentLightingType,
    pub rotation_angle: f32,
    pub rotation_matrix: Mat4,
}

impl Default for Skybox {
    fn default() -> Self {
        Skybox {
            enabled: false,
            use_ibl: false,
            use_hdr: true,
            use_diffuse_map: false,
            activated: false,
            env_lighting_type: EnvironmentLightingType::HemisphereDiffuse,
            rotation_angle: 0.0,
            rotation_matrix: Mat4::IDENTITY,
        }
    }
}

impl Skybox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn initialize(&mut self, info: &SkyboxInfo) {
        self.enabled = info.enabled;
        self.use_hdr = info.use_hdr;
        self.use_ibl = info.use_ibl;
        self.use_diffuse_map = info.apply_diffuse_map;
        self.env_lighting_type = info.env_lighting_type;
        self.rotation_angle = info.rotation_angle;
    }

    pub fn activate(&mut self) {
        self.update_rotation_matrix();
        self.activated = true;
    }

    fn update_rotation_matrix(&mut self) {
        if self.rotation_angle != 0.0 {
            let c = self.rotation_angle.cos();
            let s = self.rotation_angle.sin();
            self.rotation_matrix = Mat4::new(
                c, 0.0, -s, 0.0, 0.0, 1.0, 0.0, 0.0, s, 0.0, c, 0.0, 0.0, 0.0, 0.0, 1.0,
            );
        } else {
            self.rotation_matrix = Mat4::IDENTITY;
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, val: bool) {
        self.enabled = val;
    }

    pub fn is_use_hdr(&self) -> bool {
        self.use_hdr
    }

    pub fn set_use_hdr(&mut self, val: bool) {
        self.use_hdr = val;
    }

    pub fn is_use_ibl(&self) -> bool {
        self.use_ibl
    }

    pub fn set_use_ibl(&mut self, val: bool) {
        self.use_ibl = val;
    }

    pub fn is_use_diffuse_map(&self) -> bool {
        self.use_diffuse_map
    }

    pub fn set_use_diffuse_map(&mut self, val: bool) {
        self.use_diffuse_map = val;
    }

    pub fn set_rotation_angle(&mut self, angle: f32) {
        self.rotation_angle = angle;
        self.update_rotation_matrix();
    }

    pub fn get_rotation_angle(&self) -> f32 {
        self.rotation_angle
    }

    pub fn get_rotation_matrix(&self) -> &Mat4 {
        &self.rotation_matrix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skybox_new() {
        let skybox = Skybox::new();
        assert!(!skybox.enabled);
        assert!(skybox.use_hdr);
    }

    #[test]
    fn test_skybox_activate() {
        let mut skybox = Skybox::new();
        skybox.activate();
        assert!(skybox.activated);
    }

    #[test]
    fn test_skybox_rotation() {
        let mut skybox = Skybox::new();
        skybox.set_rotation_angle(45.0_f32.to_radians());
        assert!((skybox.rotation_angle - 45.0_f32.to_radians()).abs() < 1e-6);
    }
}
