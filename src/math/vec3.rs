use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};

use super::FLOAT_CMP_PRECISION;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub const ONE: Vec3 = Vec3 {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
    pub const UNIT_X: Vec3 = Vec3 {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };
    pub const UNIT_Y: Vec3 = Vec3 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    pub const UNIT_Z: Vec3 = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };
    pub const FORWARD: Vec3 = Vec3 {
        x: 0.0,
        y: 0.0,
        z: -1.0,
    };

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }

    pub fn from_array(array: &[f32]) -> Self {
        if array.len() >= 3 {
            Vec3 {
                x: array[0],
                y: array[1],
                z: array[2],
            }
        } else {
            Vec3::ZERO
        }
    }

    pub fn from_points(p1: &Vec3, p2: &Vec3) -> Self {
        Vec3 {
            x: p2.x - p1.x,
            y: p2.y - p1.y,
            z: p2.z - p1.z,
        }
    }

    pub fn from_color(color: u32) -> Self {
        let r = ((color >> 16) & 0xFF) as f32 / 255.0;
        let g = ((color >> 8) & 0xFF) as f32 / 255.0;
        let b = (color & 0xFF) as f32 / 255.0;
        Vec3::new(r, g, b)
    }

    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0 && self.z == 0.0
    }

    pub fn is_one(&self) -> bool {
        self.x == 1.0 && self.y == 1.0 && self.z == 1.0
    }

    pub fn angle(v1: &Vec3, v2: &Vec3) -> f32 {
        let len1 = v1.length();
        let len2 = v2.length();

        if len1 == 0.0 || len2 == 0.0 {
            return 0.0;
        }

        let dot = v1.dot(v2);
        let cos_angle = dot / (len1 * len2);
        cos_angle.clamp(-1.0, 1.0).acos()
    }

    pub fn add(&mut self, v: &Vec3) {
        self.x += v.x;
        self.y += v.y;
        self.z += v.z;
    }

    pub fn add_scalar(&mut self, xx: f32, yy: f32, zz: f32) {
        self.x += xx;
        self.y += yy;
        self.z += zz;
    }

    pub fn add_vecs(v1: &Vec3, v2: &Vec3) -> Vec3 {
        Vec3 {
            x: v1.x + v2.x,
            y: v1.y + v2.y,
            z: v1.z + v2.z,
        }
    }

    pub fn clamp(&mut self, min: &Vec3, max: &Vec3) {
        self.x = self.x.max(min.x).min(max.x);
        self.y = self.y.max(min.y).min(max.y);
        self.z = self.z.max(min.z).min(max.z);
    }

    pub fn clamp_vec(v: &Vec3, min: &Vec3, max: &Vec3) -> Vec3 {
        Vec3 {
            x: v.x.max(min.x).min(max.x),
            y: v.y.max(min.y).min(max.y),
            z: v.z.max(min.z).min(max.z),
        }
    }

    pub fn cross(&mut self, v: &Vec3) {
        let x = self.y * v.z - self.z * v.y;
        let y = self.z * v.x - self.x * v.z;
        let z = self.x * v.y - self.y * v.x;
        self.x = x;
        self.y = y;
        self.z = z;
    }

    pub fn cross_vecs(v1: &Vec3, v2: &Vec3) -> Vec3 {
        Vec3 {
            x: v1.y * v2.z - v1.z * v2.y,
            y: v1.z * v2.x - v1.x * v2.z,
            z: v1.x * v2.y - v1.y * v2.x,
        }
    }

    pub fn multiply(&mut self, v: &Vec3) {
        self.x *= v.x;
        self.y *= v.y;
        self.z *= v.z;
    }

    pub fn multiply_vecs(v1: &Vec3, v2: &Vec3) -> Vec3 {
        Vec3 {
            x: v1.x * v2.x,
            y: v1.y * v2.y,
            z: v1.z * v2.z,
        }
    }

    pub fn distance(&self, v: &Vec3) -> f32 {
        let dx = self.x - v.x;
        let dy = self.y - v.y;
        let dz = self.z - v.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn distance_squared(&self, v: &Vec3) -> f32 {
        let dx = self.x - v.x;
        let dy = self.y - v.y;
        let dz = self.z - v.z;
        dx * dx + dy * dy + dz * dz
    }

    pub fn dot(&self, v: &Vec3) -> f32 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    pub fn dot_vecs(v1: &Vec3, v2: &Vec3) -> f32 {
        v1.x * v2.x + v1.y * v2.y + v1.z * v2.z
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn negate(&mut self) {
        self.x = -self.x;
        self.y = -self.y;
        self.z = -self.z;
    }

    pub fn normalize(&mut self) {
        let len = self.length();
        if len > 0.0 {
            self.x /= len;
            self.y /= len;
            self.z /= len;
        }
    }

    pub fn get_normalized(&self) -> Vec3 {
        let len = self.length();
        if len > 0.0 {
            Vec3 {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        } else {
            *self
        }
    }

    pub fn scale(&mut self, scalar: f32) {
        self.x *= scalar;
        self.y *= scalar;
        self.z *= scalar;
    }

    pub fn set(&mut self, xx: f32, yy: f32, zz: f32) {
        self.x = xx;
        self.y = yy;
        self.z = zz;
    }

    pub fn set_from_array(&mut self, array: &[f32]) {
        if array.len() >= 3 {
            self.x = array[0];
            self.y = array[1];
            self.z = array[2];
        }
    }

    pub fn set_from_vec(&mut self, v: &Vec3) {
        self.x = v.x;
        self.y = v.y;
        self.z = v.z;
    }

    pub fn set_from_points(&mut self, p1: &Vec3, p2: &Vec3) {
        self.x = p2.x - p1.x;
        self.y = p2.y - p1.y;
        self.z = p2.z - p1.z;
    }

    pub fn set_zero(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
        self.z = 0.0;
    }

    pub fn subtract(&mut self, v: &Vec3) {
        self.x -= v.x;
        self.y -= v.y;
        self.z -= v.z;
    }

    pub fn subtract_vecs(v1: &Vec3, v2: &Vec3) -> Vec3 {
        Vec3 {
            x: v1.x - v2.x,
            y: v1.y - v2.y,
            z: v1.z - v2.z,
        }
    }

    pub fn max_vecs(v1: &Vec3, v2: &Vec3) -> Vec3 {
        Vec3 {
            x: v1.x.max(v2.x),
            y: v1.y.max(v2.y),
            z: v1.z.max(v2.z),
        }
    }

    pub fn min_vecs(v1: &Vec3, v2: &Vec3) -> Vec3 {
        Vec3 {
            x: v1.x.min(v2.x),
            y: v1.y.min(v2.y),
            z: v1.z.min(v2.z),
        }
    }

    pub fn smooth(&mut self, target: &Vec3, elapsed_time: f32, response_time: f32) {
        if response_time <= 0.0 {
            self.x = target.x;
            self.y = target.y;
            self.z = target.z;
        } else {
            let blend = 1.0 - (response_time / (response_time + elapsed_time));
            self.x = self.x * blend + target.x * (1.0 - blend);
            self.y = self.y * blend + target.y * (1.0 - blend);
            self.z = self.z * blend + target.z * (1.0 - blend);
        }
    }

    pub fn lerp(&self, target: &Vec3, alpha: f32) -> Vec3 {
        Vec3 {
            x: self.x * (1.0 - alpha) + target.x * alpha,
            y: self.y * (1.0 - alpha) + target.y * alpha,
            z: self.z * (1.0 - alpha) + target.z * alpha,
        }
    }

    pub fn approx_equals(&self, v: &Vec3, precision: f32) -> bool {
        (self.x - v.x).abs() < precision
            && (self.y - v.y).abs() < precision
            && (self.z - v.z).abs() < precision
    }

    pub fn move_towards(current: &Vec3, target: &Vec3, max_step: f32) -> Vec3 {
        let dir = *target - *current;
        let dist = dir.length();

        if dist <= max_step || dist == 0.0 {
            *target
        } else {
            let normalized = dir.get_normalized();
            *current + normalized * max_step
        }
    }

    pub fn transform_mat4(&self, m: &super::Mat4) -> Vec3 {
        let w = m.m[3] * self.x + m.m[7] * self.y + m.m[11] * self.z + m.m[15];
        let inv_w = if w.abs() > FLOAT_CMP_PRECISION { 1.0 / w } else { 1.0 };
        
        Vec3 {
            x: (m.m[0] * self.x + m.m[4] * self.y + m.m[8] * self.z + m.m[12]) * inv_w,
            y: (m.m[1] * self.x + m.m[5] * self.y + m.m[9] * self.z + m.m[13]) * inv_w,
            z: (m.m[2] * self.x + m.m[6] * self.y + m.m[10] * self.z + m.m[14]) * inv_w,
        }
    }

    pub fn transform_mat3(&self, m: &super::Mat3) -> Vec3 {
        Vec3 {
            x: m.m[0] * self.x + m.m[3] * self.y + m.m[6] * self.z,
            y: m.m[1] * self.x + m.m[4] * self.y + m.m[7] * self.z,
            z: m.m[2] * self.x + m.m[5] * self.y + m.m[8] * self.z,
        }
    }

    /// Projects this vector onto another vector
    pub fn project_onto(&self, other: &Vec3) -> Vec3 {
        let other_len_sq = other.length_squared();
        if other_len_sq == 0.0 {
            return Vec3::ZERO;
        }
        let scale = self.dot(other) / other_len_sq;
        *other * scale
    }

    /// Returns the reflection of this vector across a normal
    pub fn reflect(&self, normal: &Vec3) -> Vec3 {
        let n = normal.get_normalized();
        *self - 2.0 * self.dot(&n) * n
    }

    /// Returns a vector perpendicular to this one
    pub fn perpendicular(&self) -> Vec3 {
        if self.x.abs() < self.y.abs() {
            Vec3::cross_vecs(self, &Vec3::UNIT_X)
        } else {
            Vec3::cross_vecs(self, &Vec3::UNIT_Y)
        }
    }

    /// 用四元数变换向量
    pub fn transform_quat(&self, q: &super::Quaternion) -> Vec3 {
        let x = self.x;
        let y = self.y;
        let z = self.z;

        let qx = q.x;
        let qy = q.y;
        let qz = q.z;
        let qw = q.w;

        // calculate quat * vec
        let ix = qw * x + qy * z - qz * y;
        let iy = qw * y + qz * x - qx * z;
        let iz = qw * z + qx * y - qy * x;
        let iw = -qx * x - qy * y - qz * z;

        // calculate result * inverse quat
        Vec3 {
            x: ix * qw + iw * -qx + iy * -qz - iz * -qy,
            y: iy * qw + iw * -qy + iz * -qx - ix * -qz,
            z: iz * qw + iw * -qz + ix * -qy - iy * -qx,
        }
    }

    /// 计算向量在另一个向量上的投影
    pub fn project(&self, other: &Vec3) -> Vec3 {
        self.project_onto(other)
    }

    /// 计算向量在另一个向量上的拒绝（投影的垂直分量）
    pub fn reject(&self, other: &Vec3) -> Vec3 {
        *self - self.project_onto(other)
    }

    /// 获取两个向量之间的夹角（弧度）
    pub fn get_angle(&self, other: &Vec3) -> f32 {
        Self::angle(self, other)
    }

    /// 获取两个向量之间的有符号夹角（弧度），参考向量为参考平面
    pub fn signed_angle(from: &Vec3, to: &Vec3, axis: &Vec3) -> f32 {
        let angle = Self::angle(from, to);
        let cross = Self::cross_vecs(from, to);
        let dot = cross.dot(axis);
        
        if dot < 0.0 {
            -angle
        } else {
            angle
        }
    }
}

impl Default for Vec3 {
    fn default() -> Self {
        Vec3::ZERO
    }
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Vec3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Vec3) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f32) -> Vec3 {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

impl Div<f32> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f32) -> Vec3 {
        Vec3 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Vec3 {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl PartialOrd for Vec3 {
    fn partial_cmp(&self, rhs: &Vec3) -> Option<std::cmp::Ordering> {
        if self.x < rhs.x && self.y < rhs.y && self.z < rhs.z {
            Some(std::cmp::Ordering::Less)
        } else if self.x > rhs.x && self.y > rhs.y && self.z > rhs.z {
            Some(std::cmp::Ordering::Greater)
        } else if self.x == rhs.x && self.y == rhs.y && self.z == rhs.z {
            Some(std::cmp::Ordering::Equal)
        } else {
            None
        }
    }
}

impl std::fmt::Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vec3({:.2}, {:.2}, {:.2})", self.x, self.y, self.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    const EPSILON: f32 = 0.0001;

    fn assert_vec3_approx_eq(a: &Vec3, b: &Vec3, epsilon: f32) {
        assert!(
            (a.x - b.x).abs() < epsilon &&
            (a.y - b.y).abs() < epsilon &&
            (a.z - b.z).abs() < epsilon,
            "Vec3 not equal: {:?} != {:?}", a, b
        );
    }

    #[test]
    fn test_vec3_constants() {
        assert_eq!(Vec3::ZERO, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(Vec3::ONE, Vec3::new(1.0, 1.0, 1.0));
        assert_eq!(Vec3::UNIT_X, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(Vec3::UNIT_Y, Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(Vec3::UNIT_Z, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_vec3_new() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);
    }

    #[test]
    fn test_vec3_from_array() {
        let arr = [1.0, 2.0, 3.0];
        let v = Vec3::from_array(&arr);
        assert_eq!(v, Vec3::new(1.0, 2.0, 3.0));

        let short_arr = [1.0, 2.0];
        let v2 = Vec3::from_array(&short_arr);
        assert_eq!(v2, Vec3::ZERO);
    }

    #[test]
    fn test_vec3_from_points() {
        let p1 = Vec3::new(1.0, 2.0, 3.0);
        let p2 = Vec3::new(4.0, 6.0, 8.0);
        let v = Vec3::from_points(&p1, &p2);
        assert_eq!(v, Vec3::new(3.0, 4.0, 5.0));
    }

    #[test]
    fn test_vec3_is_zero() {
        assert!(Vec3::ZERO.is_zero());
        assert!(!Vec3::ONE.is_zero());
    }

    #[test]
    fn test_vec3_is_one() {
        assert!(Vec3::ONE.is_one());
        assert!(!Vec3::ZERO.is_one());
    }

    #[test]
    fn test_vec3_length() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        assert_eq!(v.length(), 5.0);

        let v2 = Vec3::new(1.0, 2.0, 2.0);
        assert_eq!(v2.length(), 3.0);
    }

    #[test]
    fn test_vec3_length_squared() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        assert_eq!(v.length_squared(), 25.0);
    }

    #[test]
    fn test_vec3_normalize() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        let normalized = v.get_normalized();
        assert!((normalized.length() - 1.0).abs() < EPSILON);
        assert_vec3_approx_eq(&normalized, &Vec3::new(0.6, 0.8, 0.0), EPSILON);
    }

    #[test]
    fn test_vec3_normalize_zero() {
        let v = Vec3::ZERO;
        let normalized = v.get_normalized();
        assert_eq!(normalized, Vec3::ZERO);
    }

    #[test]
    fn test_vec3_dot() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(v1.dot(&v2), 32.0);
    }

    #[test]
    fn test_vec3_cross() {
        let v1 = Vec3::UNIT_X;
        let v2 = Vec3::UNIT_Y;
        let cross = Vec3::cross_vecs(&v1, &v2);
        assert_vec3_approx_eq(&cross, &Vec3::UNIT_Z, EPSILON);

        let v3 = Vec3::new(1.0, 2.0, 3.0);
        let v4 = Vec3::new(4.0, 5.0, 6.0);
        let cross2 = Vec3::cross_vecs(&v3, &v4);
        assert_vec3_approx_eq(&cross2, &Vec3::new(-3.0, 6.0, -3.0), EPSILON);
    }

    #[test]
    fn test_vec3_distance() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 6.0, 3.0);
        assert_eq!(v1.distance(&v2), 5.0);
    }

    #[test]
    fn test_vec3_distance_squared() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 6.0, 3.0);
        assert_eq!(v1.distance_squared(&v2), 25.0);
    }

    #[test]
    fn test_vec3_angle() {
        let v1 = Vec3::UNIT_X;
        let v2 = Vec3::UNIT_Y;
        let angle = Vec3::angle(&v1, &v2);
        assert!((angle - PI / 2.0).abs() < EPSILON);

        let v3 = Vec3::new(1.0, 0.0, 0.0);
        let v4 = Vec3::new(1.0, 1.0, 0.0).get_normalized();
        let angle2 = Vec3::angle(&v3, &v4);
        assert!((angle2 - PI / 4.0).abs() < EPSILON);
    }

    #[test]
    fn test_vec3_add() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        let result = v1 + v2;
        assert_eq!(result, Vec3::new(5.0, 7.0, 9.0));
    }

    #[test]
    fn test_vec3_sub() {
        let v1 = Vec3::new(5.0, 7.0, 9.0);
        let v2 = Vec3::new(1.0, 2.0, 3.0);
        let result = v1 - v2;
        assert_eq!(result, Vec3::new(4.0, 5.0, 6.0));
    }

    #[test]
    fn test_vec3_mul_scalar() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let result = v * 2.0;
        assert_eq!(result, Vec3::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn test_vec3_div_scalar() {
        let v = Vec3::new(2.0, 4.0, 6.0);
        let result = v / 2.0;
        assert_eq!(result, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_vec3_neg() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let result = -v;
        assert_eq!(result, Vec3::new(-1.0, -2.0, -3.0));
    }

    #[test]
    fn test_vec3_lerp() {
        let v1 = Vec3::new(0.0, 0.0, 0.0);
        let v2 = Vec3::new(10.0, 10.0, 10.0);
        let result = v1.lerp(&v2, 0.5);
        assert_eq!(result, Vec3::new(5.0, 5.0, 5.0));
    }

    #[test]
    fn test_vec3_clamp() {
        let v = Vec3::new(5.0, -2.0, 10.0);
        let min = Vec3::new(0.0, 0.0, 0.0);
        let max = Vec3::new(3.0, 3.0, 3.0);
        let result = Vec3::clamp_vec(&v, &min, &max);
        assert_eq!(result, Vec3::new(3.0, 0.0, 3.0));
    }

    #[test]
    fn test_vec3_min_max() {
        let v1 = Vec3::new(1.0, 5.0, 3.0);
        let v2 = Vec3::new(3.0, 2.0, 6.0);
        let min = Vec3::min_vecs(&v1, &v2);
        let max = Vec3::max_vecs(&v1, &v2);
        assert_eq!(min, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(max, Vec3::new(3.0, 5.0, 6.0));
    }

    #[test]
    fn test_vec3_approx_equals() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(1.0001, 2.0001, 3.0001);
        assert!(v1.approx_equals(&v2, 0.001));
        assert!(!v1.approx_equals(&v2, 0.00001));
    }

    #[test]
    fn test_vec3_move_towards() {
        let current = Vec3::new(0.0, 0.0, 0.0);
        let target = Vec3::new(10.0, 0.0, 0.0);
        let result = Vec3::move_towards(&current, &target, 3.0);
        assert_vec3_approx_eq(&result, &Vec3::new(3.0, 0.0, 0.0), EPSILON);

        // When max_step is larger than distance, should return target
        let result2 = Vec3::move_towards(&current, &target, 15.0);
        assert_eq!(result2, target);
    }

    #[test]
    fn test_vec3_project_onto() {
        let v = Vec3::new(1.0, 1.0, 0.0);
        let onto = Vec3::new(1.0, 0.0, 0.0);
        let proj = v.project_onto(&onto);
        assert_vec3_approx_eq(&proj, &Vec3::new(1.0, 0.0, 0.0), EPSILON);

        let onto_zero = Vec3::ZERO;
        let proj_zero = v.project_onto(&onto_zero);
        assert_eq!(proj_zero, Vec3::ZERO);
    }

    #[test]
    fn test_vec3_reflect() {
        let v = Vec3::new(1.0, -1.0, 0.0);
        let normal = Vec3::new(0.0, 1.0, 0.0);
        let reflected = v.reflect(&normal);
        assert_vec3_approx_eq(&reflected, &Vec3::new(1.0, 1.0, 0.0), EPSILON);
    }

    #[test]
    fn test_vec3_perpendicular() {
        let v = Vec3::UNIT_X;
        let perp = v.perpendicular();
        assert!(v.dot(&perp).abs() < EPSILON); // Should be orthogonal

        let v2 = Vec3::UNIT_Y;
        let perp2 = v2.perpendicular();
        assert!(v2.dot(&perp2).abs() < EPSILON);
    }

    #[test]
    fn test_vec3_from_color() {
        let color = 0xFF8040; // RGB(255, 128, 64)
        let v = Vec3::from_color(color);
        assert!((v.x - 1.0).abs() < EPSILON);
        assert!((v.y - 0.50196).abs() < 0.01);
        assert!((v.z - 0.25098).abs() < 0.01);
    }

    #[test]
    fn test_vec3_default() {
        let v: Vec3 = Default::default();
        assert_eq!(v, Vec3::ZERO);
    }

    #[test]
    fn test_vec3_add_assign() {
        let mut v = Vec3::new(1.0, 2.0, 3.0);
        v += Vec3::new(1.0, 1.0, 1.0);
        assert_eq!(v, Vec3::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn test_vec3_sub_assign() {
        let mut v = Vec3::new(3.0, 3.0, 3.0);
        v -= Vec3::new(1.0, 1.0, 1.0);
        assert_eq!(v, Vec3::new(2.0, 2.0, 2.0));
    }

    #[test]
    fn test_vec3_mul_assign() {
        let mut v = Vec3::new(1.0, 2.0, 3.0);
        v *= 2.0;
        assert_eq!(v, Vec3::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn test_vec3_scalar_mul() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let result = 2.0 * v;
        assert_eq!(result, Vec3::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn test_vec3_partial_ord() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(2.0, 3.0, 4.0);
        let v3 = Vec3::new(1.0, 2.0, 3.0);
        
        assert_eq!(v1.partial_cmp(&v2), Some(std::cmp::Ordering::Less));
        assert_eq!(v2.partial_cmp(&v1), Some(std::cmp::Ordering::Greater));
        assert_eq!(v1.partial_cmp(&v3), Some(std::cmp::Ordering::Equal));
        
        // Incomparable case
        let v4 = Vec3::new(2.0, 1.0, 3.0);
        assert_eq!(v1.partial_cmp(&v4), None);
    }
}
