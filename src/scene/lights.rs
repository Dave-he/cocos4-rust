use crate::math::Vec3;
use super::light::Light;
use super::define::LightType;

#[derive(Debug)]
pub struct SpotLight {
    pub base: Light,
    pub position: Vec3,
    pub direction: Vec3,
    pub luminance_hdr: f32,
    pub luminance_ldr: f32,
    pub range: f32,
    pub spot_angle: f32,
    pub angle: f32,
    pub outer_angle: f32,
    pub penumbra: f32,
    pub shadow_enabled: bool,
    pub shadow_bias: f32,
    pub shadow_normal_bias: f32,
    pub shadow_near: f32,
}

impl SpotLight {
    pub fn new() -> Self {
        let mut base = Light::new(LightType::Spot);
        base.initialize();

        SpotLight {
            base,
            position: Vec3::ZERO,
            direction: Vec3::new(0.0, -1.0, 0.0),
            luminance_hdr: 1700.0,
            luminance_ldr: 1.0,
            range: 1.0,
            spot_angle: std::f32::consts::PI / 6.0,
            angle: std::f32::consts::PI / 3.0,
            outer_angle: std::f32::consts::PI / 3.0,
            penumbra: 0.05,
            shadow_enabled: false,
            shadow_bias: 0.00001,
            shadow_normal_bias: 0.0,
            shadow_near: 0.01,
        }
    }

    pub fn set_position(&mut self, pos: Vec3) {
        self.position = pos;
    }

    pub fn get_position(&self) -> &Vec3 {
        &self.position
    }

    pub fn set_direction(&mut self, dir: Vec3) {
        self.direction = dir;
    }

    pub fn get_direction(&self) -> &Vec3 {
        &self.direction
    }

    pub fn set_luminance(&mut self, value: f32) {
        self.luminance_hdr = value;
        self.luminance_ldr = value;
    }

    pub fn get_luminance_hdr(&self) -> f32 {
        self.luminance_hdr
    }

    pub fn set_luminance_hdr(&mut self, value: f32) {
        self.luminance_hdr = value;
    }

    pub fn get_luminance_ldr(&self) -> f32 {
        self.luminance_ldr
    }

    pub fn set_luminance_ldr(&mut self, value: f32) {
        self.luminance_ldr = value;
    }

    pub fn set_range(&mut self, range: f32) {
        self.range = range;
    }

    pub fn get_range(&self) -> f32 {
        self.range
    }

    pub fn set_spot_angle(&mut self, angle: f32) {
        self.spot_angle = angle;
    }

    pub fn get_spot_angle(&self) -> f32 {
        self.spot_angle
    }

    pub fn set_angle(&mut self, angle: f32) {
        self.angle = angle;
    }

    pub fn get_angle(&self) -> f32 {
        self.angle
    }

    pub fn set_outer_angle(&mut self, angle: f32) {
        self.outer_angle = angle;
    }

    pub fn get_outer_angle(&self) -> f32 {
        self.outer_angle
    }

    pub fn set_penumbra(&mut self, penumbra: f32) {
        self.penumbra = penumbra;
    }

    pub fn get_penumbra(&self) -> f32 {
        self.penumbra
    }

    pub fn set_shadow_enabled(&mut self, enabled: bool) {
        self.shadow_enabled = enabled;
    }

    pub fn is_shadow_enabled(&self) -> bool {
        self.shadow_enabled
    }
}

impl Default for SpotLight {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct PointLight {
    pub base: Light,
    pub position: Vec3,
    pub luminance_hdr: f32,
    pub luminance_ldr: f32,
    pub range: f32,
}

impl PointLight {
    pub fn new() -> Self {
        let mut base = Light::new(LightType::Point);
        base.initialize();

        PointLight {
            base,
            position: Vec3::ZERO,
            luminance_hdr: 1700.0,
            luminance_ldr: 1.0,
            range: 1.0,
        }
    }

    pub fn set_position(&mut self, pos: Vec3) {
        self.position = pos;
    }

    pub fn get_position(&self) -> &Vec3 {
        &self.position
    }

    pub fn set_luminance(&mut self, value: f32) {
        self.luminance_hdr = value;
        self.luminance_ldr = value;
    }

    pub fn get_luminance_hdr(&self) -> f32 {
        self.luminance_hdr
    }

    pub fn get_luminance_ldr(&self) -> f32 {
        self.luminance_ldr
    }

    pub fn set_range(&mut self, range: f32) {
        self.range = range;
    }

    pub fn get_range(&self) -> f32 {
        self.range
    }
}

impl Default for PointLight {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct SphereLight {
    pub base: Light,
    pub position: Vec3,
    pub luminance_hdr: f32,
    pub luminance_ldr: f32,
    pub range: f32,
    pub size: f32,
}

impl SphereLight {
    pub fn new() -> Self {
        let mut base = Light::new(super::define::LightType::Sphere);
        base.initialize();

        SphereLight {
            base,
            position: Vec3::ZERO,
            luminance_hdr: 1700.0,
            luminance_ldr: 1.0,
            range: 1.0,
            size: 0.1,
        }
    }

    pub fn set_position(&mut self, pos: Vec3) {
        self.position = pos;
    }

    pub fn get_position(&self) -> &Vec3 {
        &self.position
    }

    pub fn set_luminance(&mut self, value: f32) {
        self.luminance_hdr = value;
        self.luminance_ldr = value;
    }

    pub fn get_luminance_hdr(&self) -> f32 {
        self.luminance_hdr
    }

    pub fn get_luminance_ldr(&self) -> f32 {
        self.luminance_ldr
    }

    pub fn set_range(&mut self, range: f32) {
        self.range = range;
    }

    pub fn get_range(&self) -> f32 {
        self.range
    }

    pub fn set_size(&mut self, size: f32) {
        self.size = size;
    }

    pub fn get_size(&self) -> f32 {
        self.size
    }
}

impl Default for SphereLight {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct RangedDirectionalLight {
    pub base: Light,
    pub direction: Vec3,
    pub position: Vec3,
    pub scale: Vec3,
    pub illuminance_hdr: f32,
    pub illuminance_ldr: f32,
}

impl RangedDirectionalLight {
    pub fn new() -> Self {
        let mut base = Light::new(LightType::RangedDirectional);
        base.initialize();

        RangedDirectionalLight {
            base,
            direction: Vec3::new(0.0, -1.0, 0.0),
            position: Vec3::ZERO,
            scale: Vec3::ONE,
            illuminance_hdr: 1.0,
            illuminance_ldr: 1.0,
        }
    }

    pub fn set_direction(&mut self, dir: Vec3) {
        self.direction = dir;
    }

    pub fn get_direction(&self) -> &Vec3 {
        &self.direction
    }

    pub fn set_position(&mut self, pos: Vec3) {
        self.position = pos;
    }

    pub fn get_position(&self) -> &Vec3 {
        &self.position
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
    }

    pub fn get_scale(&self) -> &Vec3 {
        &self.scale
    }

    pub fn set_illuminance(&mut self, value: f32) {
        self.illuminance_hdr = value;
        self.illuminance_ldr = value;
    }

    pub fn get_illuminance_hdr(&self) -> f32 {
        self.illuminance_hdr
    }

    pub fn get_illuminance_ldr(&self) -> f32 {
        self.illuminance_ldr
    }
}

impl Default for RangedDirectionalLight {
    fn default() -> Self {
        Self::new()
    }
}
