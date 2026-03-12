use crate::math::Vec3;
use super::define::{PCFType, CSMLevel, CSMOptimizationMode, SUN_ILLUM};
use super::light::Light;
use super::define::LightType;

#[derive(Debug)]
pub struct DirectionalLight {
    pub base: Light,
    pub direction: Vec3,
    pub illuminance_hdr: f32,
    pub illuminance_ldr: f32,
    pub shadow_enabled: bool,
    pub shadow_pcf: PCFType,
    pub shadow_bias: f32,
    pub shadow_normal_bias: f32,
    pub shadow_saturation: f32,
    pub shadow_distance: f32,
    pub shadow_invisible_occlusion_range: f32,
    pub shadow_fixed_area: bool,
    pub shadow_near: f32,
    pub shadow_far: f32,
    pub shadow_ortho_size: f32,
    pub csm_level: CSMLevel,
    pub csm_layer_lambda: f32,
    pub csm_need_update: bool,
    pub csm_optimization_mode: CSMOptimizationMode,
    pub csm_layers_transition: bool,
    pub csm_transition_range: f32,
}

impl DirectionalLight {
    pub fn new() -> Self {
        let mut base = Light::new(LightType::Directional);
        base.initialize();

        DirectionalLight {
            base,
            direction: Vec3::new(1.0, -1.0, -1.0),
            illuminance_hdr: SUN_ILLUM,
            illuminance_ldr: 1.0,
            shadow_enabled: false,
            shadow_pcf: PCFType::Hard,
            shadow_bias: 0.0,
            shadow_normal_bias: 0.0,
            shadow_saturation: 0.75,
            shadow_distance: 50.0,
            shadow_invisible_occlusion_range: 200.0,
            shadow_fixed_area: false,
            shadow_near: 0.1,
            shadow_far: 10.0,
            shadow_ortho_size: 1.0,
            csm_level: CSMLevel::Level3,
            csm_layer_lambda: 0.75,
            csm_need_update: false,
            csm_optimization_mode: CSMOptimizationMode::RemoveDuplicates,
            csm_layers_transition: false,
            csm_transition_range: 0.05,
        }
    }

    pub fn set_direction(&mut self, dir: Vec3) {
        self.direction = dir;
    }

    pub fn get_direction(&self) -> &Vec3 {
        &self.direction
    }

    pub fn set_illuminance(&mut self, value: f32) {
        self.illuminance_hdr = value;
        self.illuminance_ldr = value;
    }

    pub fn get_illuminance_hdr(&self) -> f32 {
        self.illuminance_hdr
    }

    pub fn set_illuminance_hdr(&mut self, value: f32) {
        self.illuminance_hdr = value;
    }

    pub fn get_illuminance_ldr(&self) -> f32 {
        self.illuminance_ldr
    }

    pub fn set_illuminance_ldr(&mut self, value: f32) {
        self.illuminance_ldr = value;
    }

    pub fn set_shadow_enabled(&mut self, enabled: bool) {
        self.shadow_enabled = enabled;
    }

    pub fn is_shadow_enabled(&self) -> bool {
        self.shadow_enabled
    }

    pub fn set_shadow_pcf(&mut self, pcf: PCFType) {
        self.shadow_pcf = pcf;
    }

    pub fn get_shadow_pcf(&self) -> PCFType {
        self.shadow_pcf
    }

    pub fn set_shadow_bias(&mut self, bias: f32) {
        self.shadow_bias = bias;
    }

    pub fn get_shadow_bias(&self) -> f32 {
        self.shadow_bias
    }

    pub fn set_shadow_normal_bias(&mut self, bias: f32) {
        self.shadow_normal_bias = bias;
    }

    pub fn get_shadow_normal_bias(&self) -> f32 {
        self.shadow_normal_bias
    }

    pub fn set_shadow_saturation(&mut self, saturation: f32) {
        self.shadow_saturation = saturation;
    }

    pub fn get_shadow_saturation(&self) -> f32 {
        self.shadow_saturation
    }

    pub fn set_shadow_distance(&mut self, distance: f32) {
        self.shadow_distance = distance;
    }

    pub fn get_shadow_distance(&self) -> f32 {
        self.shadow_distance
    }

    pub fn set_csm_level(&mut self, level: CSMLevel) {
        self.csm_level = level;
    }

    pub fn get_csm_level(&self) -> CSMLevel {
        self.csm_level
    }

    pub fn set_csm_optimization_mode(&mut self, mode: CSMOptimizationMode) {
        self.csm_optimization_mode = mode;
    }

    pub fn get_csm_optimization_mode(&self) -> CSMOptimizationMode {
        self.csm_optimization_mode
    }
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self::new()
    }
}
