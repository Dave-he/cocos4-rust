use crate::math::{Color, Mat4, Vec2, Vec3};
use super::defines::{ShadowType, PCFType, CSMLevel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowSize {
    Low256 = 256,
    Medium512 = 512,
    High1024 = 1024,
    Ultra2048 = 2048,
}

impl Default for ShadowSize {
    fn default() -> Self {
        ShadowSize::Medium512
    }
}

#[derive(Debug)]
pub struct ShadowsInfo {
    pub enabled: bool,
    pub shadow_type: ShadowType,
    pub normal: Vec3,
    pub distance: f32,
    pub shadow_color: Color,
    pub saturation: f32,
    pub opacity: f32,
    pub near: f32,
    pub far: f32,
    pub aspect: f32,
    pub ortho_size: f32,
    pub size: ShadowSize,
    pub pcf_type: PCFType,
    pub bias: f32,
    pub normal_bias: f32,
    pub max_received: u32,
    pub csm_level: CSMLevel,
    pub csm_layer_lambda: f32,
    pub auto_adapt: bool,
    pub shadow_map_dirty: bool,
}

impl Default for ShadowsInfo {
    fn default() -> Self {
        ShadowsInfo {
            enabled: false,
            shadow_type: ShadowType::ShadowMap,
            normal: Vec3::new(0.0, 1.0, 0.0),
            distance: 0.0,
            shadow_color: Color::new(0, 0, 0, 76),
            saturation: 0.75,
            opacity: 1.0,
            near: 0.1,
            far: 10.0,
            aspect: 1.0,
            ortho_size: 5.0,
            size: ShadowSize::Medium512,
            pcf_type: PCFType::Hard,
            bias: 0.00001,
            normal_bias: 0.0,
            max_received: 4,
            csm_level: CSMLevel::Level1,
            csm_layer_lambda: 0.75,
            auto_adapt: true,
            shadow_map_dirty: false,
        }
    }
}

impl ShadowsInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_shadow_type(&mut self, shadow_type: ShadowType) {
        self.shadow_type = shadow_type;
        self.shadow_map_dirty = true;
    }

    pub fn set_size(&mut self, size: ShadowSize) {
        self.size = size;
        self.shadow_map_dirty = true;
    }

    pub fn get_size_u32(&self) -> u32 {
        self.size as u32
    }

    pub fn set_pcf_type(&mut self, pcf: PCFType) {
        self.pcf_type = pcf;
        self.shadow_map_dirty = true;
    }

    pub fn set_bias(&mut self, bias: f32) {
        self.bias = bias;
    }

    pub fn set_normal_bias(&mut self, bias: f32) {
        self.normal_bias = bias;
    }
}

#[derive(Debug)]
pub struct PlanarShadowInfo {
    pub shadow_color: Color,
    pub normal: Vec3,
    pub distance: f32,
    pub mat_light: Mat4,
}

impl Default for PlanarShadowInfo {
    fn default() -> Self {
        PlanarShadowInfo {
            shadow_color: Color::new(0, 0, 0, 76),
            normal: Vec3::new(0.0, 1.0, 0.0),
            distance: 0.0,
            mat_light: Mat4::IDENTITY,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shadows_info_default() {
        let info = ShadowsInfo::default();
        assert!(!info.enabled);
        assert_eq!(info.shadow_type, ShadowType::ShadowMap);
        assert_eq!(info.size, ShadowSize::Medium512);
    }

    #[test]
    fn test_shadows_info_set_size() {
        let mut info = ShadowsInfo::new();
        info.set_size(ShadowSize::High1024);
        assert_eq!(info.get_size_u32(), 1024);
        assert!(info.shadow_map_dirty);
    }

    #[test]
    fn test_shadows_info_disable() {
        let mut info = ShadowsInfo::new();
        info.enabled = true;
        info.set_enabled(false);
        assert!(!info.enabled);
    }
}
