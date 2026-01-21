use super::{Mat3, Mat4, Vec3};
use std::ops::{Add, Mul, Neg, Sub};

const FLOAT_CMP_PRECISION: f32 = 0.00001;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion {
    pub const ZERO: Quaternion = Quaternion {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };
    pub const IDENTITY: Quaternion = Quaternion {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Quaternion { x, y, z, w }
    }

    pub fn from_array(array: &[f32]) -> Self {
        if array.len() >= 4 {
            Quaternion {
                x: array[0],
                y: array[1],
                z: array[2],
                w: array[3],
            }
        } else {
            Quaternion::ZERO
        }
    }

    pub fn from_mat3(m: &Mat3) -> Self {
        let trace = m.m[0] + m.m[4] + m.m[8];
        let s = 0.5 * (1.0 + trace).sqrt();

        Quaternion {
            x: s * 0.5 * (m.m[7] - m.m[5]),
            y: s * 0.5 * (m.m[2] - m.m[6]),
            z: s * 0.5 * (m.m[3] - m.m[1]),
            w: s * 0.5,
        }
    }

    pub fn from_mat4(m: &Mat4) -> Self {
        let trace = m.m[0] + m.m[5] + m.m[10];
        let s = 0.5 * (1.0 + trace).sqrt();

        Quaternion {
            x: s * 0.5 * (m.m[7] - m.m[5]),
            y: s * 0.5 * (m.m[2] - m.m[6]),
            z: s * 0.5 * (m.m[3] - m.m[1]),
            w: s * 0.5,
        }
    }

    pub fn from_axis_angle(axis: &Vec3, angle: f32) -> Quaternion {
        let half_angle = angle / 2.0;
        let s = half_angle.sin();
        let c = half_angle.cos();

        Quaternion {
            x: axis.x * s,
            y: axis.y * s,
            z: axis.z * s,
            w: c,
        }
    }

    pub fn is_identity(&self) -> bool {
        self.x.abs() < FLOAT_CMP_PRECISION
            && self.y.abs() < FLOAT_CMP_PRECISION
            && self.z.abs() < FLOAT_CMP_PRECISION
            && (self.w - 1.0).abs() < FLOAT_CMP_PRECISION
    }

    pub fn is_zero(&self) -> bool {
        self.x.abs() < FLOAT_CMP_PRECISION
            && self.y.abs() < FLOAT_CMP_PRECISION
            && self.z.abs() < FLOAT_CMP_PRECISION
            && self.w.abs() < FLOAT_CMP_PRECISION
    }

    pub fn conjugate(&self) -> Quaternion {
        Quaternion {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }

    pub fn get_conjugated(&self) -> Quaternion {
        self.conjugate()
    }

    pub fn inverse(&self) -> Quaternion {
        let norm_sq = self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w;

        if norm_sq < FLOAT_CMP_PRECISION {
            Quaternion::ZERO
        } else {
            let inv_norm_sq = 1.0 / norm_sq;
            Quaternion {
                x: -self.x * inv_norm_sq,
                y: -self.y * inv_norm_sq,
                z: -self.z * inv_norm_sq,
                w: self.w * inv_norm_sq,
            }
        }
    }

    pub fn get_inverse(&self) -> Quaternion {
        self.inverse()
    }

    pub fn multiply(&self, other: &Quaternion) -> Quaternion {
        Quaternion {
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
        }
    }

    pub fn dot(&self, other: &Quaternion) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn normalize(&self) -> Quaternion {
        let len_sq = self.length_squared();

        if len_sq < FLOAT_CMP_PRECISION {
            return *self;
        }

        let inv_len = len_sq.sqrt().recip();

        Quaternion {
            x: self.x * inv_len,
            y: self.y * inv_len,
            z: self.z * inv_len,
            w: self.w * inv_len,
        }
    }

    pub fn get_normalized(&self) -> Quaternion {
        self.normalize()
    }

    pub fn lerp(q1: &Quaternion, q2: &Quaternion, t: f32) -> Quaternion {
        let one_minus_t = 1.0 - t;

        Quaternion {
            x: one_minus_t * q1.x + t * q2.x,
            y: one_minus_t * q1.y + t * q2.y,
            z: one_minus_t * q1.z + t * q2.z,
            w: one_minus_t * q1.w + t * q2.w,
        }
    }

    pub fn slerp(q1: &Quaternion, q2: &Quaternion, t: f32) -> Quaternion {
        let dot = q1.dot(q2);
        let mut dot_abs = dot.abs();

        if dot_abs > 1.0 {
            dot_abs = 1.0;
        }

        let theta = dot_abs.acos();
        let sin_theta = theta.sin();

        if sin_theta < FLOAT_CMP_PRECISION {
            return Quaternion::lerp(q1, q2, t);
        }

        let scale1 = ((1.0 - t) * theta).sin() / sin_theta;
        let scale2 = (t * theta).sin() / sin_theta;

        Quaternion {
            x: scale1 * q1.x + scale2 * q2.x,
            y: scale1 * q1.y + scale2 * q2.y,
            z: scale1 * q1.z + scale2 * q2.z,
            w: scale1 * q1.w + scale2 * q2.w,
        }
    }

    pub fn multiply_vec3(&self, vec: &Vec3) -> Vec3 {
        let v_quat = Quaternion {
            x: vec.x,
            y: vec.y,
            z: vec.z,
            w: 0.0,
        };

        let qv = self.multiply(&v_quat);
        let q_inv = self.get_conjugated();
        let result = qv.multiply(&q_inv);

        Vec3::new(result.x, result.y, result.z)
    }
}

impl Default for Quaternion {
    fn default() -> Self {
        Quaternion::IDENTITY
    }
}

impl Mul<f32> for Quaternion {
    type Output = Quaternion;

    fn mul(self, scalar: f32) -> Quaternion {
        Quaternion {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
            w: self.w * scalar,
        }
    }
}

impl Mul<Quaternion> for Quaternion {
    type Output = Quaternion;

    fn mul(self, other: Quaternion) -> Quaternion {
        self.multiply(&other)
    }
}

impl Add for Quaternion {
    type Output = Quaternion;

    fn add(self, other: Quaternion) -> Quaternion {
        Quaternion {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl Sub for Quaternion {
    type Output = Quaternion;

    fn sub(self, other: Quaternion) -> Quaternion {
        Quaternion {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

impl Neg for Quaternion {
    type Output = Quaternion;

    fn neg(self) -> Quaternion {
        Quaternion {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quaternion_identity() {
        let q = Quaternion::IDENTITY;
        assert!(q.is_identity());
    }

    #[test]
    fn test_quaternion_normalize() {
        let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
        let normalized = q.normalize();
        assert!((normalized.length() - 1.0).abs() < FLOAT_CMP_PRECISION);
    }
}
