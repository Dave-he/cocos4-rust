use crate::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FogType {
    #[default]
    Linear = 0,
    Exp = 1,
    ExpSquared = 2,
    Layered = 3,
    None = 4,
}

pub struct FogInfo {
    pub fog_type: FogType,
    pub fog_color: [f32; 4],
    pub is_enabled: bool,
    pub accurate: bool,
    pub fog_density: f32,
    pub fog_start: f32,
    pub fog_end: f32,
    pub fog_atten: f32,
    pub fog_top: f32,
    pub fog_range: f32,
}

impl Default for FogInfo {
    fn default() -> Self {
        FogInfo {
            fog_type: FogType::Linear,
            fog_color: [200.0 / 255.0, 200.0 / 255.0, 200.0 / 255.0, 1.0],
            is_enabled: false,
            accurate: false,
            fog_density: 0.3,
            fog_start: 0.5,
            fog_end: 300.0,
            fog_atten: 5.0,
            fog_top: 1.5,
            fog_range: 1.2,
        }
    }
}

impl FogInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_enabled(&mut self, val: bool) {
        self.is_enabled = val;
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    pub fn set_accurate(&mut self, val: bool) {
        self.accurate = val;
    }

    pub fn is_accurate(&self) -> bool {
        self.accurate
    }

    pub fn set_fog_color(&mut self, val: [f32; 4]) {
        self.fog_color = val;
    }

    pub fn get_fog_color(&self) -> &[f32; 4] {
        &self.fog_color
    }

    pub fn set_type(&mut self, val: FogType) {
        self.fog_type = val;
    }

    pub fn get_type(&self) -> FogType {
        self.fog_type
    }

    pub fn set_fog_density(&mut self, val: f32) {
        self.fog_density = val;
    }

    pub fn get_fog_density(&self) -> f32 {
        self.fog_density
    }

    pub fn set_fog_start(&mut self, val: f32) {
        self.fog_start = val;
    }

    pub fn get_fog_start(&self) -> f32 {
        self.fog_start
    }

    pub fn set_fog_end(&mut self, val: f32) {
        self.fog_end = val;
    }

    pub fn get_fog_end(&self) -> f32 {
        self.fog_end
    }

    pub fn set_fog_atten(&mut self, val: f32) {
        self.fog_atten = val;
    }

    pub fn get_fog_atten(&self) -> f32 {
        self.fog_atten
    }

    pub fn set_fog_top(&mut self, val: f32) {
        self.fog_top = val;
    }

    pub fn get_fog_top(&self) -> f32 {
        self.fog_top
    }

    pub fn set_fog_range(&mut self, val: f32) {
        self.fog_range = val;
    }

    pub fn get_fog_range(&self) -> f32 {
        self.fog_range
    }

    pub fn activate(&mut self, resource: &mut Fog) {
        resource.fog_type = self.fog_type;
        resource.fog_color = self.fog_color;
        resource.enabled = self.is_enabled;
        resource.accurate = self.accurate;
        resource.fog_density = self.fog_density;
        resource.fog_start = self.fog_start;
        resource.fog_end = self.fog_end;
        resource.fog_atten = self.fog_atten;
        resource.fog_top = self.fog_top;
        resource.fog_range = self.fog_range;
        resource.activated = true;
    }
}

pub struct Fog {
    pub enabled: bool,
    pub activated: bool,
    pub accurate: bool,
    pub fog_type: FogType,
    pub fog_color: [f32; 4],
    pub color_array: Vec3,
    pub fog_density: f32,
    pub fog_start: f32,
    pub fog_end: f32,
    pub fog_atten: f32,
    pub fog_top: f32,
    pub fog_range: f32,
}

impl Default for Fog {
    fn default() -> Self {
        Fog {
            enabled: false,
            activated: false,
            accurate: false,
            fog_type: FogType::Linear,
            fog_color: [200.0 / 255.0, 200.0 / 255.0, 200.0 / 255.0, 1.0],
            color_array: Vec3::new(0.2, 0.2, 0.2),
            fog_density: 0.3,
            fog_start: 0.5,
            fog_end: 300.0,
            fog_atten: 5.0,
            fog_top: 1.5,
            fog_range: 1.2,
        }
    }
}

impl Fog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn initialize(&mut self, info: &FogInfo) {
        self.fog_type = info.fog_type;
        self.fog_color = info.fog_color;
        self.enabled = info.is_enabled;
        self.accurate = info.accurate;
        self.fog_density = info.fog_density;
        self.fog_start = info.fog_start;
        self.fog_end = info.fog_end;
        self.fog_atten = info.fog_atten;
        self.fog_top = info.fog_top;
        self.fog_range = info.fog_range;
    }

    pub fn activate(&mut self) {
        self.update_pipeline();
        self.activated = true;
    }

    fn update_pipeline(&mut self) {
        self.color_array = Vec3::new(self.fog_color[0], self.fog_color[1], self.fog_color[2]);
    }

    pub fn set_enabled(&mut self, val: bool) {
        self.enabled = val;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_accurate(&mut self, val: bool) {
        self.accurate = val;
    }

    pub fn is_accurate(&self) -> bool {
        self.accurate
    }

    pub fn set_fog_color(&mut self, val: [f32; 4]) {
        self.fog_color = val;
        self.update_pipeline();
    }

    pub fn get_fog_color(&self) -> &[f32; 4] {
        &self.fog_color
    }

    pub fn get_type(&self) -> FogType {
        self.fog_type
    }

    pub fn set_type(&mut self, val: FogType) {
        self.fog_type = val;
    }

    pub fn get_fog_density(&self) -> f32 {
        self.fog_density
    }

    pub fn set_fog_density(&mut self, val: f32) {
        self.fog_density = val;
    }

    pub fn get_fog_start(&self) -> f32 {
        self.fog_start
    }

    pub fn set_fog_start(&mut self, val: f32) {
        self.fog_start = val;
    }

    pub fn get_fog_end(&self) -> f32 {
        self.fog_end
    }

    pub fn set_fog_end(&mut self, val: f32) {
        self.fog_end = val;
    }

    pub fn get_fog_atten(&self) -> f32 {
        self.fog_atten
    }

    pub fn set_fog_atten(&mut self, val: f32) {
        self.fog_atten = val;
    }

    pub fn get_fog_top(&self) -> f32 {
        self.fog_top
    }

    pub fn set_fog_top(&mut self, val: f32) {
        self.fog_top = val;
    }

    pub fn get_fog_range(&self) -> f32 {
        self.fog_range
    }

    pub fn set_fog_range(&mut self, val: f32) {
        self.fog_range = val;
    }

    pub fn get_color_array(&self) -> &Vec3 {
        &self.color_array
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fog_new() {
        let fog = Fog::new();
        assert!(!fog.enabled);
        assert_eq!(fog.fog_type, FogType::Linear);
    }

    #[test]
    fn test_fog_activate() {
        let mut fog = Fog::new();
        fog.activate();
        assert!(fog.activated);
    }

    #[test]
    fn test_fog_set_color() {
        let mut fog = Fog::new();
        fog.set_fog_color([0.5, 0.5, 0.5, 1.0]);
        assert_eq!(fog.fog_color[0], 0.5);
    }
}
