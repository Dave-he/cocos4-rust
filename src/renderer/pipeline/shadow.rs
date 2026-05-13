/****************************************************************************
Rust port of Cocos Creator Shadow System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::math::{Color, Mat4, Vec3, Vec4};
use super::defines::{ShadowType, PCFType, CSMLevel};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShadowSize {
    Low256 = 256,
    #[default]
    Medium512 = 512,
    High1024 = 1024,
    Ultra2048 = 2048,
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
    pub mat_light: Mat4,
    pub light_dir: Vec3,
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
            mat_light: Mat4::IDENTITY,
            light_dir: Vec3::new(0.0, -1.0, 0.0),
        }
    }
}

impl ShadowsInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.shadow_map_dirty = true;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_shadow_type(&mut self, shadow_type: ShadowType) {
        self.shadow_type = shadow_type;
        self.shadow_map_dirty = true;
    }

    pub fn get_shadow_type(&self) -> ShadowType {
        self.shadow_type
    }

    pub fn set_size(&mut self, size: ShadowSize) {
        self.size = size;
        self.shadow_map_dirty = true;
    }

    pub fn get_size_u32(&self) -> u32 {
        self.size as u32
    }

    pub fn get_size(&self) -> ShadowSize {
        self.size
    }

    pub fn set_pcf_type(&mut self, pcf: PCFType) {
        self.pcf_type = pcf;
        self.shadow_map_dirty = true;
    }

    pub fn get_pcf_type(&self) -> PCFType {
        self.pcf_type
    }

    pub fn set_bias(&mut self, bias: f32) {
        self.bias = bias;
    }

    pub fn get_bias(&self) -> f32 {
        self.bias
    }

    pub fn set_normal_bias(&mut self, bias: f32) {
        self.normal_bias = bias;
    }

    pub fn get_normal_bias(&self) -> f32 {
        self.normal_bias
    }

    pub fn set_distance(&mut self, distance: f32) {
        self.distance = distance;
    }

    pub fn get_distance(&self) -> f32 {
        self.distance
    }

    pub fn set_normal(&mut self, normal: Vec3) {
        self.normal = normal;
    }

    pub fn get_normal(&self) -> Vec3 {
        self.normal
    }

    pub fn set_shadow_color(&mut self, color: Color) {
        self.shadow_color = color;
    }

    pub fn get_shadow_color(&self) -> Color {
        self.shadow_color
    }

    pub fn set_saturation(&mut self, saturation: f32) {
        self.saturation = saturation;
    }

    pub fn get_saturation(&self) -> f32 {
        self.saturation
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity;
    }

    pub fn get_opacity(&self) -> f32 {
        self.opacity
    }

    pub fn set_near(&mut self, near: f32) {
        self.near = near;
    }

    pub fn get_near(&self) -> f32 {
        self.near
    }

    pub fn set_far(&mut self, far: f32) {
        self.far = far;
    }

    pub fn get_far(&self) -> f32 {
        self.far
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    pub fn get_aspect(&self) -> f32 {
        self.aspect
    }

    pub fn set_ortho_size(&mut self, size: f32) {
        self.ortho_size = size;
    }

    pub fn get_ortho_size(&self) -> f32 {
        self.ortho_size
    }

    pub fn set_max_received(&mut self, max: u32) {
        self.max_received = max;
    }

    pub fn get_max_received(&self) -> u32 {
        self.max_received
    }

    pub fn set_csm_level(&mut self, level: CSMLevel) {
        self.csm_level = level;
        self.shadow_map_dirty = true;
    }

    pub fn get_csm_level(&self) -> CSMLevel {
        self.csm_level
    }

    pub fn set_csm_layer_lambda(&mut self, lambda: f32) {
        self.csm_layer_lambda = lambda;
    }

    pub fn get_csm_layer_lambda(&self) -> f32 {
        self.csm_layer_lambda
    }

    pub fn set_auto_adapt(&mut self, auto: bool) {
        self.auto_adapt = auto;
    }

    pub fn is_auto_adapt(&self) -> bool {
        self.auto_adapt
    }

    pub fn is_shadow_map_dirty(&self) -> bool {
        self.shadow_map_dirty
    }

    pub fn reset_shadow_map_dirty(&mut self) {
        self.shadow_map_dirty = false;
    }

    pub fn set_mat_light(&mut self, mat: Mat4) {
        self.mat_light = mat;
    }

    pub fn get_mat_light(&self) -> &Mat4 {
        &self.mat_light
    }

    pub fn set_light_dir(&mut self, dir: Vec3) {
        self.light_dir = dir;
    }

    pub fn get_light_dir(&self) -> Vec3 {
        self.light_dir
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

#[derive(Debug, Clone, Default)]
pub struct ShadowUBOInfo {
    pub mat_light_view: Mat4,
    pub mat_light_view_proj: Mat4,
    pub shadow_inv_proj_depth_info: Vec4,
    pub shadow_proj_depth_info: Vec4,
    pub shadow_proj_info: Vec4,
    pub shadow_nfls_info: Vec4,
    pub shadow_whpb_info: Vec4,
    pub shadow_lpnn_info: Vec4,
    pub shadow_color: Vec4,
    pub planar_nd_info: Vec4,
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
        assert!(info.is_shadow_map_dirty());
    }

    #[test]
    fn test_shadows_info_disable() {
        let mut info = ShadowsInfo::new();
        info.enabled = true;
        info.set_enabled(false);
        assert!(!info.is_enabled());
    }

    #[test]
    fn test_shadows_info_csm() {
        let mut info = ShadowsInfo::new();
        info.set_csm_level(CSMLevel::Level4);
        assert_eq!(info.get_csm_level(), CSMLevel::Level4);
    }

    #[test]
    fn test_shadows_info_distance() {
        let mut info = ShadowsInfo::new();
        info.set_distance(100.0);
        assert_eq!(info.get_distance(), 100.0);
    }

    #[test]
    fn test_shadows_info_reset_dirty() {
        let mut info = ShadowsInfo::new();
        info.set_shadow_type(ShadowType::Planar);
        assert!(info.is_shadow_map_dirty());
        info.reset_shadow_map_dirty();
        assert!(!info.is_shadow_map_dirty());
    }

    #[test]
    fn test_planar_shadow_default() {
        let planar = PlanarShadowInfo::default();
        assert_eq!(planar.normal, Vec3::new(0.0, 1.0, 0.0));
    }
}
