use crate::math::Vec3;
use super::define::{LightType, CAMERA_DEFAULT_MASK};

#[derive(Debug)]
pub struct Light {
    pub name: String,
    pub light_type: LightType,
    pub color: Vec3,
    pub color_temperature: f32,
    pub color_temperature_rgb: Vec3,
    pub use_color_temperature: bool,
    pub baked: bool,
    pub visibility: u32,
    pub node_uuid: Option<String>,
}

impl Light {
    pub fn new(light_type: LightType) -> Self {
        Light {
            name: String::new(),
            light_type,
            color: Vec3::new(1.0, 1.0, 1.0),
            color_temperature: 6550.0,
            color_temperature_rgb: Vec3::ZERO,
            use_color_temperature: false,
            baked: false,
            visibility: CAMERA_DEFAULT_MASK,
            node_uuid: None,
        }
    }

    pub fn initialize(&mut self) {
        self.color = Vec3::new(1.0, 1.0, 1.0);
        self.color_temperature = 6550.0;
    }

    pub fn set_color(&mut self, color: Vec3) {
        self.color = color;
    }

    pub fn get_color(&self) -> &Vec3 {
        &self.color
    }

    pub fn set_color_temperature(&mut self, kelvin: f32) {
        self.color_temperature = kelvin;
        self.color_temperature_rgb = Light::color_temperature_to_rgb(kelvin);
    }

    pub fn get_color_temperature(&self) -> f32 {
        self.color_temperature
    }

    pub fn set_use_color_temperature(&mut self, value: bool) {
        self.use_color_temperature = value;
    }

    pub fn is_use_color_temperature(&self) -> bool {
        self.use_color_temperature
    }

    pub fn get_color_temperature_rgb(&self) -> &Vec3 {
        &self.color_temperature_rgb
    }

    pub fn set_baked(&mut self, baked: bool) {
        self.baked = baked;
    }

    pub fn is_baked(&self) -> bool {
        self.baked
    }

    pub fn set_visibility(&mut self, visibility: u32) {
        self.visibility = visibility;
    }

    pub fn get_visibility(&self) -> u32 {
        self.visibility
    }

    pub fn get_type(&self) -> LightType {
        self.light_type
    }

    pub fn nt2lm(size: f32) -> f32 {
        std::f32::consts::PI * size * size
    }

    pub fn color_temperature_to_rgb(kelvin: f32) -> Vec3 {
        let k = kelvin / 100.0;
        let r;
        let g;
        let b;

        #[allow(clippy::excessive_precision)]
        if k <= 66.0 {
            r = 1.0;
            g = (99.4708025861 * k.ln() - 161.1195681661) / 255.0;
        } else {
            r = (329.698727446 * (k - 60.0).powf(-0.1332047592)) / 255.0;
            g = (288.1221695283 * (k - 60.0).powf(-0.0755148492)) / 255.0;
        }

        #[allow(clippy::excessive_precision)]
        if k >= 66.0 {
            b = 1.0;
        } else if k <= 19.0 {
            b = 0.0;
        } else {
            b = (138.5177312231 * (k - 10.0).ln() - 305.0447927307) / 255.0;
        }

        Vec3::new(
            r.clamp(0.0, 1.0),
            g.clamp(0.0, 1.0),
            b.clamp(0.0, 1.0),
        )
    }
}
