use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EasingMethod {
    Linear,
    QuadIn,
    QuadOut,
    QuadInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    QuartIn,
    QuartOut,
    QuartInOut,
    QuintIn,
    QuintOut,
    QuintInOut,
    SineIn,
    SineOut,
    SineInOut,
    ExpoIn,
    ExpoOut,
    ExpoInOut,
    CircIn,
    CircOut,
    CircInOut,
    BounceIn,
    BounceOut,
    BounceInOut,
    ElasticIn,
    ElasticOut,
    ElasticInOut,
    BackIn,
    BackOut,
    BackInOut,
    Smooth,
    Fade,
}

impl EasingMethod {
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            EasingMethod::Linear => t,
            EasingMethod::QuadIn => t * t,
            EasingMethod::QuadOut => t * (2.0 - t),
            EasingMethod::QuadInOut => {
                if t < 0.5 { 2.0 * t * t }
                else { -1.0 + (4.0 - 2.0 * t) * t }
            }
            EasingMethod::CubicIn => t * t * t,
            EasingMethod::CubicOut => {
                let s = t - 1.0;
                s * s * s + 1.0
            }
            EasingMethod::CubicInOut => {
                if t < 0.5 { 4.0 * t * t * t }
                else {
                    let s = t - 1.0;
                    (2.0 * s) * (2.0 * s) * (2.0 * s) / 2.0 + 1.0
                }
            }
            EasingMethod::QuartIn => t * t * t * t,
            EasingMethod::QuartOut => {
                let s = t - 1.0;
                1.0 - s * s * s * s
            }
            EasingMethod::QuartInOut => {
                if t < 0.5 { 8.0 * t * t * t * t }
                else {
                    let s = t - 1.0;
                    1.0 - 8.0 * s * s * s * s
                }
            }
            EasingMethod::QuintIn => t * t * t * t * t,
            EasingMethod::QuintOut => {
                let s = t - 1.0;
                s * s * s * s * s + 1.0
            }
            EasingMethod::QuintInOut => {
                if t < 0.5 { 16.0 * t * t * t * t * t }
                else {
                    let s = t - 1.0;
                    16.0 * s * s * s * s * s + 1.0
                }
            }
            EasingMethod::SineIn => 1.0 - (t * PI / 2.0).cos(),
            EasingMethod::SineOut => (t * PI / 2.0).sin(),
            EasingMethod::SineInOut => -(( PI * t).cos() - 1.0) / 2.0,
            EasingMethod::ExpoIn => {
                if t == 0.0 { 0.0 } else { (2.0f32).powf(10.0 * t - 10.0) }
            }
            EasingMethod::ExpoOut => {
                if t == 1.0 { 1.0 } else { 1.0 - (2.0f32).powf(-10.0 * t) }
            }
            EasingMethod::ExpoInOut => {
                if t == 0.0 { return 0.0; }
                if t == 1.0 { return 1.0; }
                if t < 0.5 { (2.0f32).powf(20.0 * t - 10.0) / 2.0 }
                else { (2.0 - (2.0f32).powf(-20.0 * t + 10.0)) / 2.0 }
            }
            EasingMethod::CircIn => 1.0 - (1.0 - t * t).sqrt(),
            EasingMethod::CircOut => (1.0 - (t - 1.0) * (t - 1.0)).sqrt(),
            EasingMethod::CircInOut => {
                if t < 0.5 {
                    (1.0 - (1.0 - (2.0 * t) * (2.0 * t)).sqrt()) / 2.0
                } else {
                    ((1.0 - (-2.0 * t + 2.0) * (-2.0 * t + 2.0)).sqrt() + 1.0) / 2.0
                }
            }
            EasingMethod::BounceIn => 1.0 - EasingMethod::BounceOut.apply(1.0 - t),
            EasingMethod::BounceOut => {
                if t < 1.0 / 2.75 {
                    7.5625 * t * t
                } else if t < 2.0 / 2.75 {
                    let t = t - 1.5 / 2.75;
                    7.5625 * t * t + 0.75
                } else if t < 2.5 / 2.75 {
                    let t = t - 2.25 / 2.75;
                    7.5625 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / 2.75;
                    7.5625 * t * t + 0.984375
                }
            }
            EasingMethod::BounceInOut => {
                if t < 0.5 {
                    (1.0 - EasingMethod::BounceOut.apply(1.0 - 2.0 * t)) / 2.0
                } else {
                    (1.0 + EasingMethod::BounceOut.apply(2.0 * t - 1.0)) / 2.0
                }
            }
            EasingMethod::ElasticIn => {
                if t == 0.0 { return 0.0; }
                if t == 1.0 { return 1.0; }
                let c4 = (2.0 * PI) / 3.0;
                -(2.0f32).powf(10.0 * t - 10.0) * ((10.0 * t - 10.75) * c4).sin()
            }
            EasingMethod::ElasticOut => {
                if t == 0.0 { return 0.0; }
                if t == 1.0 { return 1.0; }
                let c4 = (2.0 * PI) / 3.0;
                (2.0f32).powf(-10.0 * t) * ((10.0 * t - 0.75) * c4).sin() + 1.0
            }
            EasingMethod::ElasticInOut => {
                if t == 0.0 { return 0.0; }
                if t == 1.0 { return 1.0; }
                let c5 = (2.0 * PI) / 4.5;
                if t < 0.5 {
                    -(2.0f32).powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * c5).sin() / 2.0
                } else {
                    (2.0f32).powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * c5).sin() / 2.0 + 1.0
                }
            }
            EasingMethod::BackIn => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                c3 * t * t * t - c1 * t * t
            }
            EasingMethod::BackOut => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                let s = t - 1.0;
                1.0 + c3 * s * s * s + c1 * s * s
            }
            EasingMethod::BackInOut => {
                let c1 = 1.70158;
                let c2 = c1 * 1.525;
                if t < 0.5 {
                    ((2.0 * t) * (2.0 * t) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
                } else {
                    ((2.0 * t - 2.0) * (2.0 * t - 2.0) * ((c2 + 1.0) * (2.0 * t - 2.0) + c2) + 2.0) / 2.0
                }
            }
            EasingMethod::Smooth => t * t * (3.0 - 2.0 * t),
            EasingMethod::Fade => t * t * t * (t * (6.0 * t - 15.0) + 10.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear() {
        assert!((EasingMethod::Linear.apply(0.0) - 0.0).abs() < 1e-5);
        assert!((EasingMethod::Linear.apply(0.5) - 0.5).abs() < 1e-5);
        assert!((EasingMethod::Linear.apply(1.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_all_start_at_0_end_at_1() {
        let methods = [
            EasingMethod::Linear,
            EasingMethod::QuadIn, EasingMethod::QuadOut, EasingMethod::QuadInOut,
            EasingMethod::CubicIn, EasingMethod::CubicOut, EasingMethod::CubicInOut,
            EasingMethod::SineIn, EasingMethod::SineOut, EasingMethod::SineInOut,
            EasingMethod::BounceIn, EasingMethod::BounceOut, EasingMethod::BounceInOut,
            EasingMethod::BackIn, EasingMethod::BackOut,
            EasingMethod::Smooth, EasingMethod::Fade,
        ];
        for m in &methods {
            let v0 = m.apply(0.0);
            let v1 = m.apply(1.0);
            assert!(v0.abs() < 1e-4, "{:?} at t=0 was {}", m, v0);
            assert!((v1 - 1.0).abs() < 1e-4, "{:?} at t=1 was {}", m, v1);
        }
    }

    #[test]
    fn test_quad_in() {
        assert!((EasingMethod::QuadIn.apply(0.5) - 0.25).abs() < 1e-5);
    }

    #[test]
    fn test_bounce_out() {
        let v = EasingMethod::BounceOut.apply(1.0);
        assert!((v - 1.0).abs() < 1e-5);
    }
}
