use crate::math::{Mat4, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShadowType {
    #[default]
    None = 0,
    Planar = 1,
    ShadowMap = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowSize {
    Low256x256 = 256,
    Medium512x512 = 512,
    High1024x1024 = 1024,
    Ultra2048x2048 = 2048,
}

pub struct ShadowsInfo {
    pub enabled: bool,
    pub shadow_type: ShadowType,
    pub shadow_color: [f32; 4],
    pub normal: Vec3,
    pub distance: f32,
    pub plane_bias: f32,
    pub max_received: u32,
    pub size: [f32; 2],
}

impl Default for ShadowsInfo {
    fn default() -> Self {
        ShadowsInfo {
            enabled: false,
            shadow_type: ShadowType::Planar,
            shadow_color: [0.0, 0.0, 76.0 / 255.0, 1.0],
            normal: Vec3::new(0.0, 1.0, 0.0),
            distance: 0.0,
            plane_bias: 1.0,
            max_received: 4,
            size: [1024.0, 1024.0],
        }
    }
}

impl ShadowsInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_enabled(&mut self, val: bool) {
        self.enabled = val;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_type(&mut self, val: ShadowType) {
        self.shadow_type = val;
    }

    pub fn get_type(&self) -> ShadowType {
        self.shadow_type
    }

    pub fn set_shadow_color(&mut self, val: [f32; 4]) {
        self.shadow_color = val;
    }

    pub fn get_shadow_color(&self) -> &[f32; 4] {
        &self.shadow_color
    }

    pub fn set_normal(&mut self, val: Vec3) {
        self.normal = val;
    }

    pub fn get_normal(&self) -> &Vec3 {
        &self.normal
    }

    pub fn set_distance(&mut self, val: f32) {
        self.distance = val;
    }

    pub fn get_distance(&self) -> f32 {
        self.distance
    }

    pub fn set_plane_bias(&mut self, val: f32) {
        self.plane_bias = val;
    }

    pub fn get_plane_bias(&self) -> f32 {
        self.plane_bias
    }

    pub fn set_max_received(&mut self, val: u32) {
        self.max_received = val;
    }

    pub fn get_max_received(&self) -> u32 {
        self.max_received
    }

    pub fn set_shadow_map_size(&mut self, value: f32) {
        self.size = [value, value];
    }

    pub fn get_shadow_map_size(&self) -> f32 {
        self.size[0]
    }

    pub fn activate(&mut self, resource: &mut Shadows) {
        resource.enabled = self.enabled;
        resource.shadow_type = self.shadow_type;
        resource.shadow_color = self.shadow_color;
        resource.normal = self.normal;
        resource.distance = self.distance;
        resource.plane_bias = self.plane_bias;
        resource.max_received = self.max_received;
        resource.size = self.size;
    }
}

pub struct Shadows {
    pub enabled: bool,
    pub shadow_map_dirty: bool,
    pub shadow_type: ShadowType,
    pub shadow_color: [f32; 4],
    pub shadow_color_4f: [f32; 4],
    pub normal: Vec3,
    pub distance: f32,
    pub plane_bias: f32,
    pub max_received: u32,
    pub size: [f32; 2],
    pub mat_light: Mat4,
}

impl Shadows {
    pub const MAX_FAR: f32 = 2000.0;
    pub const COEFFICIENT_OF_EXPANSION: f32 = 0.1;

    pub fn new() -> Self {
        Shadows {
            enabled: false,
            shadow_map_dirty: false,
            shadow_type: ShadowType::None,
            shadow_color: [0.0, 0.0, 76.0 / 255.0, 1.0],
            shadow_color_4f: [0.0, 0.0, 0.0, 76.0 / 255.0],
            normal: Vec3::new(0.0, 1.0, 0.0),
            distance: 0.0,
            plane_bias: 1.0,
            max_received: 4,
            size: [1024.0, 1024.0],
            mat_light: Mat4::IDENTITY,
        }
    }

    pub fn initialize(&mut self, info: &ShadowsInfo) {
        self.enabled = info.enabled;
        self.shadow_type = info.shadow_type;
        self.shadow_color = info.shadow_color;
        self.normal = info.normal;
        self.distance = info.distance;
        self.plane_bias = info.plane_bias;
        self.max_received = info.max_received;
        self.size = info.size;
        self.shadow_color_4f = [
            info.shadow_color[0],
            info.shadow_color[1],
            info.shadow_color[2],
            info.shadow_color[3],
        ];
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, val: bool) {
        self.enabled = val;
    }

    pub fn get_normal(&self) -> &Vec3 {
        &self.normal
    }

    pub fn set_normal(&mut self, val: Vec3) {
        self.normal = val;
    }

    pub fn get_distance(&self) -> f32 {
        self.distance
    }

    pub fn set_distance(&mut self, val: f32) {
        self.distance = val;
    }

    pub fn get_plane_bias(&self) -> f32 {
        self.plane_bias
    }

    pub fn set_plane_bias(&mut self, val: f32) {
        self.plane_bias = val;
    }

    pub fn get_shadow_color(&self) -> &[f32; 4] {
        &self.shadow_color
    }

    pub fn set_shadow_color(&mut self, color: [f32; 4]) {
        self.shadow_color = color;
        self.shadow_color_4f = color;
    }

    pub fn get_shadow_color_4f(&self) -> &[f32; 4] {
        &self.shadow_color_4f
    }

    pub fn get_type(&self) -> ShadowType {
        self.shadow_type
    }

    pub fn set_type(&mut self, val: ShadowType) {
        self.shadow_type = val;
    }

    pub fn get_size(&self) -> &[f32; 2] {
        &self.size
    }

    pub fn set_size(&mut self, val: [f32; 2]) {
        self.size = val;
        self.shadow_map_dirty = true;
    }

    pub fn set_shadow_map_size(&mut self, value: f32) {
        self.size = [value, value];
        self.shadow_map_dirty = true;
    }

    pub fn get_shadow_map_size(&self) -> f32 {
        self.size[0]
    }

    pub fn is_shadow_map_dirty(&self) -> bool {
        self.shadow_map_dirty
    }

    pub fn set_shadow_map_dirty(&mut self, val: bool) {
        self.shadow_map_dirty = val;
    }

    pub fn get_mat_light(&self) -> &Mat4 {
        &self.mat_light
    }

    pub fn get_mat_light_mut(&mut self) -> &mut Mat4 {
        &mut self.mat_light
    }

    pub fn set_max_received(&mut self, val: u32) {
        self.max_received = val;
    }

    pub fn get_max_received(&self) -> u32 {
        self.max_received
    }
}

impl Default for Shadows {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shadows_new() {
        let shadows = Shadows::new();
        assert!(!shadows.enabled);
        assert_eq!(shadows.shadow_type, ShadowType::None);
    }

    #[test]
    fn test_shadows_set_enabled() {
        let mut shadows = Shadows::new();
        shadows.set_enabled(true);
        assert!(shadows.is_enabled());
    }

    #[test]
    fn test_shadows_set_type() {
        let mut shadows = Shadows::new();
        shadows.set_type(ShadowType::ShadowMap);
        assert_eq!(shadows.get_type(), ShadowType::ShadowMap);
    }

    #[test]
    fn test_shadows_set_size() {
        let mut shadows = Shadows::new();
        shadows.set_shadow_map_size(512.0);
        assert_eq!(shadows.get_shadow_map_size(), 512.0);
        assert!(shadows.is_shadow_map_dirty());
    }
}
