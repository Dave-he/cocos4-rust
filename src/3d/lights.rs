use crate::math::{Color, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LightType {
    #[default]
    Directional,
    Point,
    Spot,
    Sphere,
    RangedDirectional,
}

#[derive(Debug, Clone)]
pub struct Light {
    pub light_type: LightType,
    pub enabled: bool,
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub direction: Vec3,
    pub position: Vec3,
    pub spot_angle: f32,
    pub spot_inner_angle: f32,
    pub cast_shadow: bool,
    pub shadow_bias: f32,
    pub shadow_normal_bias: f32,
    pub shadow_near_plane: f32,
    pub shadow_distance: f32,
    pub frustum_size: f32,
    pub luminance: f32,
    pub term: u32,
}

impl Default for Light {
    fn default() -> Self {
        Self::directional()
    }
}

impl Light {
    pub fn directional() -> Self {
        Self {
            light_type: LightType::Directional,
            enabled: true,
            color: Color::new(255, 255, 255, 255),
            intensity: 1.0,
            range: 0.0,
            direction: Vec3::new(-1.0, -1.0, -1.0),
            position: Vec3::ZERO,
            spot_angle: 0.0,
            spot_inner_angle: 0.0,
            cast_shadow: false,
            shadow_bias: 0.005,
            shadow_normal_bias: 0.005,
            shadow_near_plane: 0.1,
            shadow_distance: 70.0,
            frustum_size: 0.0,
            luminance: 0.0,
            term: 0,
        }
    }

    pub fn point() -> Self {
        let mut l = Self::directional();
        l.light_type = LightType::Point;
        l.range = 10.0;
        l
    }

    pub fn spot() -> Self {
        let mut l = Self::directional();
        l.light_type = LightType::Spot;
        l.range = 10.0;
        l.spot_angle = 45.0_f32.to_radians();
        l.spot_inner_angle = 30.0_f32.to_radians();
        l
    }

    pub fn sphere() -> Self {
        let mut l = Self::directional();
        l.light_type = LightType::Sphere;
        l.range = 10.0;
        l
    }

    pub fn ranged_directional() -> Self {
        let mut l = Self::directional();
        l.light_type = LightType::RangedDirectional;
        l
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity.max(0.0);
    }

    pub fn set_range(&mut self, range: f32) {
        self.range = range.max(0.0);
    }

    pub fn set_direction(&mut self, dir: Vec3) {
        let len = (dir.x * dir.x + dir.y * dir.y + dir.z * dir.z).sqrt();
        if len > f32::EPSILON {
            self.direction = Vec3::new(dir.x / len, dir.y / len, dir.z / len);
        }
    }

    pub fn set_position(&mut self, pos: Vec3) {
        self.position = pos;
    }

    pub fn set_spot_angle(&mut self, angle: f32) {
        self.spot_angle = angle.clamp(0.0, std::f32::consts::FRAC_PI_2);
    }

    pub fn set_spot_inner_angle(&mut self, angle: f32) {
        self.spot_inner_angle = angle.clamp(0.0, self.spot_angle);
    }

    pub fn enable_shadow(&mut self, enabled: bool) {
        self.cast_shadow = enabled;
    }

    pub fn is_directional(&self) -> bool {
        matches!(self.light_type, LightType::Directional | LightType::RangedDirectional)
    }

    pub fn is_point_like(&self) -> bool {
        matches!(self.light_type, LightType::Point | LightType::Sphere)
    }

    pub fn is_spot(&self) -> bool {
        self.light_type == LightType::Spot
    }

    pub fn get_attenuation(&self, distance: f32) -> f32 {
        if self.range <= 0.0 || self.is_directional() {
            return 1.0;
        }
        if distance >= self.range {
            return 0.0;
        }
        let d = distance / self.range;
        1.0 - d * d
    }

    pub fn get_spot_attenuation(&self, cos_angle: f32) -> f32 {
        if !self.is_spot() || self.spot_angle <= 0.0 {
            return 1.0;
        }
        let cos_outer = self.spot_angle.cos();
        let cos_inner = self.spot_inner_angle.cos();
        if cos_angle <= cos_outer {
            return 0.0;
        }
        if cos_angle >= cos_inner {
            return 1.0;
        }
        (cos_angle - cos_outer) / (cos_inner - cos_outer)
    }

    pub fn get_forward(&self) -> Vec3 {
        self.direction
    }
}

pub struct LightList {
    lights: Vec<Light>,
}

impl Default for LightList {
    fn default() -> Self { Self::new() }
}

impl LightList {
    pub fn new() -> Self {
        Self { lights: Vec::new() }
    }

    pub fn add(&mut self, light: Light) -> usize {
        self.lights.push(light);
        self.lights.len() - 1
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.lights.len() {
            self.lights.remove(index);
        }
    }

    pub fn get(&self, index: usize) -> Option<&Light> {
        self.lights.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Light> {
        self.lights.get_mut(index)
    }

    pub fn count(&self) -> usize {
        self.lights.len()
    }

    pub fn get_directional_lights(&self) -> Vec<&Light> {
        self.lights.iter().filter(|l| l.enabled && l.is_directional()).collect()
    }

    pub fn get_point_lights(&self) -> Vec<&Light> {
        self.lights.iter().filter(|l| l.enabled && l.is_point_like()).collect()
    }

    pub fn get_spot_lights(&self) -> Vec<&Light> {
        self.lights.iter().filter(|l| l.enabled && l.is_spot()).collect()
    }

    pub fn get_shadow_casters(&self) -> Vec<&Light> {
        self.lights.iter().filter(|l| l.enabled && l.cast_shadow).collect()
    }

    pub fn clear(&mut self) {
        self.lights.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &Light> {
        self.lights.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directional_light() {
        let l = Light::directional();
        assert_eq!(l.light_type, LightType::Directional);
        assert!(l.enabled);
        assert!(l.is_directional());
        assert!(!l.is_point_like());
        assert!(!l.is_spot());
    }

    #[test]
    fn test_point_light() {
        let l = Light::point();
        assert_eq!(l.light_type, LightType::Point);
        assert!(l.is_point_like());
        assert!((l.range - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_spot_light() {
        let l = Light::spot();
        assert_eq!(l.light_type, LightType::Spot);
        assert!(l.is_spot());
        assert!((l.spot_angle - 45.0_f32.to_radians()).abs() < 1e-6);
    }

    #[test]
    fn test_set_intensity() {
        let mut l = Light::directional();
        l.set_intensity(2.5);
        assert!((l.intensity - 2.5).abs() < 1e-6);
        l.set_intensity(-1.0);
        assert_eq!(l.intensity, 0.0);
    }

    #[test]
    fn test_set_direction_normalized() {
        let mut l = Light::directional();
        l.set_direction(Vec3::new(10.0, 0.0, 0.0));
        assert!((l.direction.x - 1.0).abs() < 1e-6);
        assert!((l.direction.y).abs() < 1e-6);
        assert!((l.direction.z).abs() < 1e-6);
    }

    #[test]
    fn test_set_zero_direction() {
        let mut l = Light::directional();
        let orig = l.direction;
        l.set_direction(Vec3::ZERO);
        assert_eq!(l.direction, orig);
    }

    #[test]
    fn test_attenuation_directional() {
        let l = Light::directional();
        assert!((l.get_attenuation(100.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_attenuation_point() {
        let l = Light::point();
        assert!((l.get_attenuation(0.0) - 1.0).abs() < 1e-6);
        assert!((l.get_attenuation(10.0)).abs() < 1e-6);
        assert!(l.get_attenuation(5.0) > 0.0 && l.get_attenuation(5.0) < 1.0);
    }

    #[test]
    fn test_spot_attenuation() {
        let mut l = Light::spot();
        l.spot_angle = 45.0_f32.to_radians();
        l.spot_inner_angle = 30.0_f32.to_radians();
        assert!((l.get_spot_attenuation(1.0) - 1.0).abs() < 1e-6);
        assert!((l.get_spot_attenuation(0.0)).abs() < 1e-6);
        let mid = l.get_spot_attenuation(37.5_f32.to_radians().cos());
        assert!(mid > 0.0 && mid < 1.0);
    }

    #[test]
    fn test_spot_attenuation_non_spot() {
        let l = Light::directional();
        assert!((l.get_spot_attenuation(0.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_light_list() {
        let mut list = LightList::new();
        assert_eq!(list.count(), 0);
        list.add(Light::directional());
        list.add(Light::point());
        list.add(Light::spot());
        assert_eq!(list.count(), 3);
        assert_eq!(list.get_directional_lights().len(), 1);
        assert_eq!(list.get_point_lights().len(), 1);
        assert_eq!(list.get_spot_lights().len(), 1);
    }

    #[test]
    fn test_light_list_shadow_casters() {
        let mut list = LightList::new();
        let mut l1 = Light::directional();
        l1.cast_shadow = true;
        let l2 = Light::point();
        list.add(l1);
        list.add(l2);
        assert_eq!(list.get_shadow_casters().len(), 1);
    }

    #[test]
    fn test_light_list_remove_clear() {
        let mut list = LightList::new();
        list.add(Light::directional());
        list.add(Light::point());
        list.remove(0);
        assert_eq!(list.count(), 1);
        list.clear();
        assert_eq!(list.count(), 0);
    }

    #[test]
    fn test_set_spot_angle_clamped() {
        let mut l = Light::spot();
        l.set_spot_angle(180.0_f32.to_radians());
        assert!(l.spot_angle <= std::f32::consts::FRAC_PI_2);
        l.set_spot_angle(-1.0);
        assert_eq!(l.spot_angle, 0.0);
    }

    #[test]
    fn test_set_spot_inner_angle_clamped() {
        let mut l = Light::spot();
        l.spot_angle = 45.0_f32.to_radians();
        l.set_spot_inner_angle(30.0_f32.to_radians());
        assert!((l.spot_inner_angle - 30.0_f32.to_radians()).abs() < 1e-6);
        l.set_spot_inner_angle(60.0_f32.to_radians());
        assert!(l.spot_inner_angle <= l.spot_angle);
    }

    #[test]
    fn test_enable_shadow() {
        let mut l = Light::directional();
        l.enable_shadow(true);
        assert!(l.cast_shadow);
        l.enable_shadow(false);
        assert!(!l.cast_shadow);
    }

    #[test]
    fn test_sphere_light() {
        let l = Light::sphere();
        assert_eq!(l.light_type, LightType::Sphere);
        assert!(l.is_point_like());
    }

    #[test]
    fn test_ranged_directional() {
        let l = Light::ranged_directional();
        assert_eq!(l.light_type, LightType::RangedDirectional);
        assert!(l.is_directional());
    }
}
