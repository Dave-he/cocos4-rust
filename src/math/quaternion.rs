use super::{Mat3, Mat4, Vec3};
use std::ops::{Add, Mul, Neg, Sub};

const FLOAT_CMP_PRECISION: f32 = 0.00001;
const HALF_TO_RAD: f32 = 0.5 * std::f32::consts::PI / 180.0;

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
        let m00 = m.m[0];
        let m01 = m.m[1];
        let m02 = m.m[2];
        let m10 = m.m[3];
        let m11 = m.m[4];
        let m12 = m.m[5];
        let m20 = m.m[6];
        let m21 = m.m[7];
        let m22 = m.m[8];

        let four_x_sq_minus1 = m00 - m11 - m22;
        let four_y_sq_minus1 = m11 - m00 - m22;
        let four_z_sq_minus1 = m22 - m00 - m11;
        let four_w_sq_minus1 = m00 + m11 + m22;

        let mut biggest_index = 0;
        let mut four_biggest_sq_minus1 = four_w_sq_minus1;
        if four_x_sq_minus1 > four_biggest_sq_minus1 {
            four_biggest_sq_minus1 = four_x_sq_minus1;
            biggest_index = 1;
        }
        if four_y_sq_minus1 > four_biggest_sq_minus1 {
            four_biggest_sq_minus1 = four_y_sq_minus1;
            biggest_index = 2;
        }
        if four_z_sq_minus1 > four_biggest_sq_minus1 {
            four_biggest_sq_minus1 = four_z_sq_minus1;
            biggest_index = 3;
        }

        let biggest_val = (four_biggest_sq_minus1 + 1.0).sqrt() * 0.5;
        let mult = 0.25 / biggest_val;

        match biggest_index {
            0 => Quaternion {
                w: biggest_val,
                x: (m12 - m21) * mult,
                y: (m20 - m02) * mult,
                z: (m01 - m10) * mult,
            },
            1 => Quaternion {
                w: (m12 - m21) * mult,
                x: biggest_val,
                y: (m01 + m10) * mult,
                z: (m20 + m02) * mult,
            },
            2 => Quaternion {
                w: (m20 - m02) * mult,
                x: (m01 + m10) * mult,
                y: biggest_val,
                z: (m12 + m21) * mult,
            },
            3 => Quaternion {
                w: (m01 - m10) * mult,
                x: (m20 + m02) * mult,
                y: (m12 + m21) * mult,
                z: biggest_val,
            },
            _ => Quaternion::IDENTITY,
        }
    }

    pub fn from_mat4(m: &Mat4) -> Self {
        let mat3 = Mat3::new(
            m.m[0], m.m[1], m.m[2],
            m.m[4], m.m[5], m.m[6],
            m.m[8], m.m[9], m.m[10],
        );
        Quaternion::from_mat3(&mat3)
    }

    pub fn from_axis_angle(axis: &Vec3, angle: f32) -> Quaternion {
        let mut normal = *axis;
        normal.normalize();
        let half_angle = angle * 0.5;
        let s = half_angle.sin();

        Quaternion {
            x: normal.x * s,
            y: normal.y * s,
            z: normal.z * s,
            w: half_angle.cos(),
        }
    }

    pub fn from_angle_z(z_degrees: f32) -> Quaternion {
        let z = z_degrees * HALF_TO_RAD;
        Quaternion {
            x: 0.0,
            y: 0.0,
            z: z.sin(),
            w: z.cos(),
        }
    }

    pub fn from_euler(x_deg: f32, y_deg: f32, z_deg: f32) -> Quaternion {
        let x = x_deg * HALF_TO_RAD;
        let y = y_deg * HALF_TO_RAD;
        let z = z_deg * HALF_TO_RAD;
        let sx = x.sin();
        let cx = x.cos();
        let sy = y.sin();
        let cy = y.cos();
        let sz = z.sin();
        let cz = z.cos();

        Quaternion {
            x: sx * cy * cz + cx * sy * sz,
            y: cx * sy * cz + sx * cy * sz,
            z: cx * cy * sz - sx * sy * cz,
            w: cx * cy * cz - sx * sy * sz,
        }
    }

    pub fn to_euler(&self, outer_z: bool) -> Vec3 {
        let x = self.x;
        let y = self.y;
        let z = self.z;
        let w = self.w;
        let test = x * y + z * w;
        let r2d = 180.0 / std::f32::consts::PI;

        let (bank, heading, attitude);

        if test > 0.499999 {
            bank = 0.0;
            heading = 2.0 * x.atan2(w) * r2d;
            attitude = 90.0;
        } else if test < -0.499999 {
            bank = 0.0;
            heading = -2.0 * x.atan2(w) * r2d;
            attitude = -90.0;
        } else {
            let sqx = x * x;
            let sqy = y * y;
            let sqz = z * z;
            bank = (2.0 * x * w - 2.0 * y * z).atan2(1.0 - 2.0 * sqx - 2.0 * sqz) * r2d;
            heading = (2.0 * y * w - 2.0 * x * z).atan2(1.0 - 2.0 * sqy - 2.0 * sqz) * r2d;
            attitude = (2.0 * test).asin() * r2d;
        }

        if outer_z {
            let sgn = |v: f32| -> f32 {
                if v >= 0.0 { 1.0 } else { -1.0 }
            };
            Vec3::new(
                -180.0 * sgn(bank + 1e-6) + bank,
                -180.0 * sgn(heading + 1e-6) + heading,
                180.0 * sgn(attitude + 1e-6) - attitude,
            )
        } else {
            Vec3::new(bank, heading, attitude)
        }
    }

    pub fn from_view_up(view: &Vec3, up: Option<&Vec3>) -> Quaternion {
        let m = Mat3::from_view_up(view, up);
        let mut q = Quaternion::from_mat3(&m);
        q.normalize_self();
        q
    }

    pub fn to_axis_angle(&self) -> (Vec3, f32) {
        let mut q = self.normalize();
        let mut axis = Vec3::new(q.x, q.y, q.z);
        axis.normalize();
        let angle = 2.0 * q.w.acos();
        (axis, angle)
    }

    pub fn angle(a: &Quaternion, b: &Quaternion) -> f32 {
        let dot = Quaternion::dot_static(a, b).abs().min(1.0);
        dot.acos() * 2.0
    }

    pub fn rotate_towards(from: &Quaternion, to: &Quaternion, max_step: f32) -> Quaternion {
        let angle = Quaternion::angle(from, to);
        if angle == 0.0 {
            return *to;
        }
        let t = (max_step / (angle * 180.0 / std::f32::consts::PI)).min(1.0);
        Quaternion::slerp(from, to, t)
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

    pub fn approx_equals(&self, other: &Quaternion, precision: Option<f32>) -> bool {
        let p = precision.unwrap_or(FLOAT_CMP_PRECISION);
        (self.x - other.x).abs() < p
            && (self.y - other.y).abs() < p
            && (self.z - other.z).abs() < p
            && (self.w - other.w).abs() < p
    }

    pub fn set(&mut self, x: f32, y: f32, z: f32, w: f32) {
        self.x = x;
        self.y = y;
        self.z = z;
        self.w = w;
    }

    pub fn set_from_array(&mut self, array: &[f32]) {
        if array.len() >= 4 {
            self.x = array[0];
            self.y = array[1];
            self.z = array[2];
            self.w = array[3];
        }
    }

    pub fn set_from_mat4(&mut self, m: &Mat4) {
        *self = Quaternion::from_mat4(m);
    }

    pub fn set_from_axis_angle(&mut self, axis: &Vec3, angle: f32) {
        *self = Quaternion::from_axis_angle(axis, angle);
    }

    pub fn set_from_quaternion(&mut self, q: &Quaternion) {
        *self = *q;
    }

    pub fn set_identity(&mut self) {
        *self = Quaternion::IDENTITY;
    }

    pub fn conjugate(&self) -> Quaternion {
        Quaternion {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }

    pub fn conjugate_self(&mut self) {
        self.x = -self.x;
        self.y = -self.y;
        self.z = -self.z;
    }

    pub fn get_conjugated(&self) -> Quaternion {
        self.conjugate()
    }

    pub fn inverse(&self) -> Quaternion {
        let dot = self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w;
        let inv_dot = if dot > 0.0 { 1.0 / dot } else { 0.0 };

        Quaternion {
            x: -self.x * inv_dot,
            y: -self.y * inv_dot,
            z: -self.z * inv_dot,
            w: self.w * inv_dot,
        }
    }

    pub fn inverse_self(&mut self) {
        *self = self.inverse();
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

    pub fn multiply_self(&mut self, other: &Quaternion) {
        *self = self.multiply(other);
    }

    pub fn dot(&self, other: &Quaternion) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    pub fn dot_static(a: &Quaternion, b: &Quaternion) -> f32 {
        a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn normalize(&self) -> Quaternion {
        let len = self.length_squared();
        if len > 0.0 {
            let inv_len = (1.0 / len).sqrt();
            Quaternion {
                x: self.x * inv_len,
                y: self.y * inv_len,
                z: self.z * inv_len,
                w: self.w * inv_len,
            }
        } else {
            Quaternion::ZERO
        }
    }

    pub fn normalize_self(&mut self) {
        *self = self.normalize();
    }

    pub fn get_normalized(&self) -> Quaternion {
        self.normalize()
    }

    pub fn lerp(q1: &Quaternion, q2: &Quaternion, t: f32) -> Quaternion {
        let t1 = 1.0 - t;
        Quaternion {
            x: t1 * q1.x + t * q2.x,
            y: t1 * q1.y + t * q2.y,
            z: t1 * q1.z + t * q2.z,
            w: t1 * q1.w + t * q2.w,
        }
    }

    pub fn slerp(a: &Quaternion, b: &Quaternion, t: f32) -> Quaternion {
        let mut bx = b.x;
        let mut by = b.y;
        let mut bz = b.z;
        let mut bw = b.w;

        let mut cosom = a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w;
        if cosom < 0.0 {
            cosom = -cosom;
            bx = -bx;
            by = -by;
            bz = -bz;
            bw = -bw;
        }

        let (scale0, scale1);
        if (1.0 - cosom) > 0.000001 {
            let omega = cosom.acos();
            let sinom = omega.sin();
            scale0 = ((1.0 - t) * omega).sin() / sinom;
            scale1 = (t * omega).sin() / sinom;
        } else {
            scale0 = 1.0 - t;
            scale1 = t;
        }

        Quaternion {
            x: scale0 * a.x + scale1 * bx,
            y: scale0 * a.y + scale1 * by,
            z: scale0 * a.z + scale1 * bz,
            w: scale0 * a.w + scale1 * bw,
        }
    }

    pub fn sqlerp(
        a: &Quaternion,
        b: &Quaternion,
        c: &Quaternion,
        d: &Quaternion,
        t: f32,
    ) -> Quaternion {
        let q1 = Quaternion::slerp(a, d, t);
        let q2 = Quaternion::slerp(b, c, t);
        Quaternion::slerp(&q1, &q2, 2.0 * t * (1.0 - t))
    }

    pub fn multiply_vec3(&self, vec: &Vec3) -> Vec3 {
        let v_quat = Quaternion {
            x: vec.x,
            y: vec.y,
            z: vec.z,
            w: 0.0,
        };
        let qv = self.multiply(&v_quat);
        let q_conj = self.get_conjugated();
        let result = qv.multiply(&q_conj);
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

    #[test]
    fn test_quaternion_from_mat3_w_dominant() {
        let q_orig = Quaternion::from_axis_angle(&Vec3::new(0.0, 1.0, 0.0), 0.3);
        let m = Mat3::from_quat(&q_orig);
        let q_back = Quaternion::from_mat3(&m);
        assert!(q_orig.approx_equals(&q_back, Some(0.001)));
    }

    #[test]
    fn test_quaternion_from_mat3_x_dominant() {
        let q_orig = Quaternion::from_axis_angle(&Vec3::new(1.0, 0.0, 0.0), std::f32::consts::PI);
        let m = Mat3::from_quat(&q_orig);
        let q_back = Quaternion::from_mat3(&m);
        let dot = q_orig.dot(&q_back).abs();
        assert!((dot - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_quaternion_from_euler_roundtrip() {
        let q = Quaternion::from_euler(30.0, 45.0, 60.0);
        let euler = q.to_euler(false);
        let q2 = Quaternion::from_euler(euler.x, euler.y, euler.z);
        let dot = q.dot(&q2).abs();
        assert!((dot - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_quaternion_slerp() {
        let q1 = Quaternion::IDENTITY;
        let q2 = Quaternion::from_axis_angle(&Vec3::new(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_2);
        let mid = Quaternion::slerp(&q1, &q2, 0.5);
        let expected = Quaternion::from_axis_angle(&Vec3::new(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_4);
        assert!(mid.approx_equals(&expected, Some(0.001)));
    }

    #[test]
    fn test_quaternion_inverse() {
        let q = Quaternion::from_euler(30.0, 45.0, 60.0);
        let qi = q.inverse();
        let product = q.multiply(&qi);
        assert!(product.approx_equals(&Quaternion::IDENTITY, Some(0.001)));
    }

    #[test]
    fn test_quaternion_to_axis_angle() {
        let axis = Vec3::new(0.0, 1.0, 0.0);
        let angle = 1.0_f32;
        let q = Quaternion::from_axis_angle(&axis, angle);
        let (out_axis, out_angle) = q.to_axis_angle();
        assert!((out_angle - angle).abs() < 0.001);
        assert!((out_axis.y - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_quaternion_angle() {
        let q1 = Quaternion::IDENTITY;
        let q2 = Quaternion::from_axis_angle(&Vec3::new(0.0, 1.0, 0.0), 1.0);
        let a = Quaternion::angle(&q1, &q2);
        assert!((a - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_quaternion_from_angle_z() {
        let q = Quaternion::from_angle_z(90.0);
        let expected = Quaternion::from_euler(0.0, 0.0, 90.0);
        assert!(q.approx_equals(&expected, Some(0.001)));
    }
}
