pub mod color;
pub mod geometry;
pub mod mat3;
pub mod mat4;
pub mod mathf;
pub mod quaternion;
pub mod vec2;
pub mod vec3;
pub mod vec4;

pub use color::Color;
pub use geometry::{Rect, Size};
pub use mat3::Mat3;
pub use mat4::Mat4;
pub use mathf::{
    lerp as mathf_lerp, clamp as mathf_clamp, clamp01 as mathf_clamp01,
    smooth_step, smoother_step, ping_pong, repeat as mathf_repeat,
    remap, move_towards, deg_to_rad, rad_to_deg, approximately, sign,
    BezierCurve2D, AnimationCurve, catmull_rom, hermite,
};
pub use quaternion::Quaternion;
pub use vec2::Vec2;
pub use vec2::Vec2 as Point;
pub use vec3::Vec3;
pub use vec4::Vec4;

/// Floating point comparison precision
pub const FLOAT_CMP_PRECISION: f32 = 0.00001;

/// Compares two floating point numbers for approximate equality
pub fn approx(a: f32, b: f32, epsilon: Option<f32>) -> bool {
    let epsilon = epsilon.unwrap_or(FLOAT_CMP_PRECISION);
    (a - b).abs() < epsilon
}

/// Clamps a value between min and max
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Clamps a value between 0.0 and 1.0
pub fn clamp01(value: f32) -> f32 {
    clamp(value, 0.0, 1.0)
}

/// Linearly interpolates between two values
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Converts degrees to radians
pub fn to_radian(degrees: f32) -> f32 {
    degrees * std::f32::consts::PI / 180.0
}

/// Converts radians to degrees
pub fn to_degree(radians: f32) -> f32 {
    radians * 180.0 / std::f32::consts::PI
}
