use crate::math::Color;

#[derive(Debug, Clone, PartialEq)]
pub struct GradientKey<T: Clone + PartialEq> {
    pub time: f32,
    pub value: T,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorOverLifetime {
    pub enabled: bool,
    pub keys: Vec<GradientKey<Color>>,
}

impl ColorOverLifetime {
    pub fn new() -> Self {
        ColorOverLifetime {
            enabled: false,
            keys: vec![
                GradientKey { time: 0.0, value: Color::WHITE },
                GradientKey { time: 1.0, value: Color::WHITE },
            ],
        }
    }

    pub fn evaluate(&self, t: f32) -> Color {
        if self.keys.is_empty() { return Color::WHITE; }
        if t <= self.keys[0].time { return self.keys[0].value; }
        if t >= self.keys.last().unwrap().time { return self.keys.last().unwrap().value; }
        for i in 0..self.keys.len() - 1 {
            let k0 = &self.keys[i];
            let k1 = &self.keys[i + 1];
            if t >= k0.time && t <= k1.time {
                let alpha = (t - k0.time) / (k1.time - k0.time);
                return lerp_color(k0.value, k1.value, alpha);
            }
        }
        Color::WHITE
    }
}

impl Default for ColorOverLifetime {
    fn default() -> Self {
        Self::new()
    }
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        (a.r as f32 + (b.r as f32 - a.r as f32) * t) as u8,
        (a.g as f32 + (b.g as f32 - a.g as f32) * t) as u8,
        (a.b as f32 + (b.b as f32 - a.b as f32) * t) as u8,
        (a.a as f32 + (b.a as f32 - a.a as f32) * t) as u8,
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct SizeOverLifetime {
    pub enabled: bool,
    pub keys: Vec<GradientKey<f32>>,
    pub separate_axes: bool,
}

impl SizeOverLifetime {
    pub fn new() -> Self {
        SizeOverLifetime {
            enabled: false,
            keys: vec![
                GradientKey { time: 0.0, value: 1.0 },
                GradientKey { time: 1.0, value: 1.0 },
            ],
            separate_axes: false,
        }
    }

    pub fn evaluate(&self, t: f32) -> f32 {
        if self.keys.is_empty() { return 1.0; }
        if t <= self.keys[0].time { return self.keys[0].value; }
        if t >= self.keys.last().unwrap().time { return self.keys.last().unwrap().value; }
        for i in 0..self.keys.len() - 1 {
            let k0 = &self.keys[i];
            let k1 = &self.keys[i + 1];
            if t >= k0.time && t <= k1.time {
                let alpha = (t - k0.time) / (k1.time - k0.time);
                return k0.value + (k1.value - k0.value) * alpha;
            }
        }
        1.0
    }
}

impl Default for SizeOverLifetime {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VelocityOverLifetime {
    pub enabled: bool,
    pub x_keys: Vec<GradientKey<f32>>,
    pub y_keys: Vec<GradientKey<f32>>,
    pub z_keys: Vec<GradientKey<f32>>,
}

impl VelocityOverLifetime {
    pub fn new() -> Self {
        VelocityOverLifetime {
            enabled: false,
            x_keys: vec![GradientKey { time: 0.0, value: 0.0 }, GradientKey { time: 1.0, value: 0.0 }],
            y_keys: vec![GradientKey { time: 0.0, value: 0.0 }, GradientKey { time: 1.0, value: 0.0 }],
            z_keys: vec![GradientKey { time: 0.0, value: 0.0 }, GradientKey { time: 1.0, value: 0.0 }],
        }
    }

    fn eval_channel(keys: &[GradientKey<f32>], t: f32) -> f32 {
        if keys.is_empty() { return 0.0; }
        if t <= keys[0].time { return keys[0].value; }
        if t >= keys.last().unwrap().time { return keys.last().unwrap().value; }
        for i in 0..keys.len() - 1 {
            let k0 = &keys[i];
            let k1 = &keys[i + 1];
            if t >= k0.time && t <= k1.time {
                let alpha = (t - k0.time) / (k1.time - k0.time);
                return k0.value + (k1.value - k0.value) * alpha;
            }
        }
        0.0
    }

    pub fn evaluate(&self, t: f32) -> (f32, f32, f32) {
        (
            Self::eval_channel(&self.x_keys, t),
            Self::eval_channel(&self.y_keys, t),
            Self::eval_channel(&self.z_keys, t),
        )
    }
}

impl Default for VelocityOverLifetime {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RotationOverLifetime {
    pub enabled: bool,
    pub keys: Vec<GradientKey<f32>>,
}

impl RotationOverLifetime {
    pub fn new() -> Self {
        RotationOverLifetime {
            enabled: false,
            keys: vec![
                GradientKey { time: 0.0, value: 0.0 },
                GradientKey { time: 1.0, value: 0.0 },
            ],
        }
    }

    pub fn evaluate(&self, t: f32) -> f32 {
        SizeOverLifetime { enabled: false, keys: self.keys.clone(), separate_axes: false }.evaluate(t)
    }
}

impl Default for RotationOverLifetime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_over_lifetime_white() {
        let col = ColorOverLifetime::new();
        let c = col.evaluate(0.5);
        assert_eq!(c, Color::WHITE);
    }

    #[test]
    fn test_color_over_lifetime_interpolation() {
        let mut col = ColorOverLifetime::new();
        col.keys = vec![
            GradientKey { time: 0.0, value: Color::new(0, 0, 0, 255) },
            GradientKey { time: 1.0, value: Color::new(255, 255, 255, 255) },
        ];
        let c = col.evaluate(0.5);
        assert!((c.r as f32 - 127.5).abs() < 2.0);
    }

    #[test]
    fn test_size_over_lifetime() {
        let mut sol = SizeOverLifetime::new();
        sol.keys = vec![
            GradientKey { time: 0.0, value: 0.0 },
            GradientKey { time: 1.0, value: 2.0 },
        ];
        let s = sol.evaluate(0.5);
        assert!((s - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_velocity_over_lifetime() {
        let vol = VelocityOverLifetime::new();
        let (vx, vy, vz) = vol.evaluate(0.5);
        assert!((vx - 0.0).abs() < 1e-5);
        assert!((vy - 0.0).abs() < 1e-5);
        assert!((vz - 0.0).abs() < 1e-5);
    }
}
