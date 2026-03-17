use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};

use super::FLOAT_CMP_PRECISION;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };
    pub const ONE: Vec2 = Vec2 { x: 1.0, y: 1.0 };
    pub const UNIT_X: Vec2 = Vec2 { x: 1.0, y: 0.0 };
    pub const UNIT_Y: Vec2 = Vec2 { x: 0.0, y: 1.0 };
    pub const ANCHOR_MIDDLE: Vec2 = Vec2 { x: 0.5, y: 0.5 };
    pub const ANCHOR_BOTTOM_LEFT: Vec2 = Vec2 { x: 0.0, y: 0.0 };
    pub const ANCHOR_TOP_LEFT: Vec2 = Vec2 { x: 0.0, y: 1.0 };
    pub const ANCHOR_BOTTOM_RIGHT: Vec2 = Vec2 { x: 1.0, y: 0.0 };
    pub const ANCHOR_TOP_RIGHT: Vec2 = Vec2 { x: 1.0, y: 1.0 };
    pub const ANCHOR_MIDDLE_RIGHT: Vec2 = Vec2 { x: 1.0, y: 0.5 };
    pub const ANCHOR_MIDDLE_LEFT: Vec2 = Vec2 { x: 0.0, y: 0.5 };
    pub const ANCHOR_MIDDLE_TOP: Vec2 = Vec2 { x: 0.5, y: 1.0 };
    pub const ANCHOR_MIDDLE_BOTTOM: Vec2 = Vec2 { x: 0.5, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }

    pub fn from_array(array: &[f32]) -> Self {
        if array.len() >= 2 {
            Vec2 {
                x: array[0],
                y: array[1],
            }
        } else {
            Vec2::ZERO
        }
    }

    pub fn from_points(p1: &Vec2, p2: &Vec2) -> Self {
        Vec2 {
            x: p2.x - p1.x,
            y: p2.y - p1.y,
        }
    }

    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }

    pub fn is_one(&self) -> bool {
        self.x == 1.0 && self.y == 1.0
    }

    pub fn angle(v1: &Vec2, v2: &Vec2) -> f32 {
        (v2.y.atan2(v2.x) - v1.y.atan2(v1.x)).abs()
    }

    pub fn add(&mut self, v: &Vec2) {
        self.x += v.x;
        self.y += v.y;
    }

    pub fn add_vecs(v1: &Vec2, v2: &Vec2) -> Vec2 {
        Vec2 {
            x: v1.x + v2.x,
            y: v1.y + v2.y,
        }
    }

    pub fn clamp(&mut self, min: &Vec2, max: &Vec2) {
        self.x = self.x.max(min.x).min(max.x);
        self.y = self.y.max(min.y).min(max.y);
    }

    pub fn clamp_vec(v: &Vec2, min: &Vec2, max: &Vec2) -> Vec2 {
        Vec2 {
            x: v.x.max(min.x).min(max.x),
            y: v.y.max(min.y).min(max.y),
        }
    }

    pub fn distance(&self, v: &Vec2) -> f32 {
        let dx = self.x - v.x;
        let dy = self.y - v.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn distance_squared(&self, v: &Vec2) -> f32 {
        let dx = self.x - v.x;
        let dy = self.y - v.y;
        dx * dx + dy * dy
    }

    pub fn dot(&self, v: &Vec2) -> f32 {
        self.x * v.x + self.y * v.y
    }

    pub fn dot_vecs(v1: &Vec2, v2: &Vec2) -> f32 {
        v1.x * v2.x + v1.y * v2.y
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn negate(&mut self) {
        self.x = -self.x;
        self.y = -self.y;
    }

    pub fn normalize(&mut self) {
        let len = self.length();
        if len > 0.0 {
            self.x /= len;
            self.y /= len;
        }
    }

    pub fn get_normalized(&self) -> Vec2 {
        let len = self.length();
        if len > 0.0 {
            Vec2 {
                x: self.x / len,
                y: self.y / len,
            }
        } else {
            *self
        }
    }

    pub fn scale(&mut self, scalar: f32) {
        self.x *= scalar;
        self.y *= scalar;
    }

    pub fn scale_vec(&self, scale: &Vec2) -> Vec2 {
        Vec2 {
            x: self.x * scale.x,
            y: self.y * scale.y,
        }
    }

    pub fn rotate(&mut self, point: &Vec2, angle: f32) {
        let sin_angle = angle.sin();
        let cos_angle = angle.cos();

        let dx = self.x - point.x;
        let dy = self.y - point.y;

        self.x = cos_angle * dx - sin_angle * dy + point.x;
        self.y = sin_angle * dx + cos_angle * dy + point.y;
    }

    pub fn set(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn set_from_array(&mut self, array: &[f32]) {
        if array.len() >= 2 {
            self.x = array[0];
            self.y = array[1];
        }
    }

    pub fn set_from_vec(&mut self, v: &Vec2) {
        self.x = v.x;
        self.y = v.y;
    }

    pub fn set_from_points(&mut self, p1: &Vec2, p2: &Vec2) {
        self.x = p2.x - p1.x;
        self.y = p2.y - p1.y;
    }

    pub fn set_zero(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
    }

    pub fn subtract(&mut self, v: &Vec2) {
        self.x -= v.x;
        self.y -= v.y;
    }

    pub fn subtract_vecs(v1: &Vec2, v2: &Vec2) -> Vec2 {
        Vec2 {
            x: v1.x - v2.x,
            y: v1.y - v2.y,
        }
    }

    pub fn smooth(&mut self, target: &Vec2, elapsed_time: f32, response_time: f32) {
        if response_time <= 0.0 {
            self.x = target.x;
            self.y = target.y;
        } else {
            let blend = 1.0 - (response_time / (response_time + elapsed_time));
            self.x = self.x * blend + target.x * (1.0 - blend);
            self.y = self.y * blend + target.y * (1.0 - blend);
        }
    }

    pub fn equals(&self, target: &Vec2) -> bool {
        (self.x - target.x).abs() < FLOAT_CMP_PRECISION
            && (self.y - target.y).abs() < FLOAT_CMP_PRECISION
    }

    pub fn fuzzy_equals(&self, b: &Vec2, var: f32) -> bool {
        self.x < b.x + var && self.x > b.x - var && self.y < b.y + var && self.y > b.y - var
    }

    pub fn get_angle(&self) -> f32 {
        self.y.atan2(self.x)
    }

    pub fn get_angle_with(&self, other: &Vec2) -> f32 {
        other.get_angle() - self.get_angle()
    }

    pub fn cross(&self, other: &Vec2) -> f32 {
        self.x * other.y - self.y * other.x
    }

    pub fn get_perp(&self) -> Vec2 {
        Vec2 {
            x: -self.y,
            y: self.x,
        }
    }

    pub fn get_midpoint(&self, other: &Vec2) -> Vec2 {
        Vec2 {
            x: (self.x + other.x) * 0.5,
            y: (self.y + other.y) * 0.5,
        }
    }

    pub fn get_clamp_point(&self, min_inclusive: &Vec2, max_inclusive: &Vec2) -> Vec2 {
        Vec2 {
            x: self.x.max(min_inclusive.x).min(max_inclusive.x),
            y: self.y.max(min_inclusive.y).min(max_inclusive.y),
        }
    }

    pub fn get_rperp(&self) -> Vec2 {
        Vec2 {
            x: self.y,
            y: -self.x,
        }
    }

    pub fn project(&self, other: &Vec2) -> Vec2 {
        let dot_self = self.dot(other);
        let dot_other = other.dot(other);
        *other * (dot_self / dot_other)
    }

    /// 计算向量在另一个向量上的拒绝（投影的垂直分量）
    pub fn reject(&self, other: &Vec2) -> Vec2 {
        *self - self.project(other)
    }

    /// 用 4x4 矩阵变换向量
    pub fn transform_mat4(&self, m: &super::Mat4) -> Vec2 {
        let x = self.x;
        let y = self.y;
        
        let w = m.m[3] * x + m.m[7] * y + m.m[15];
        let inv_w = if w.abs() > FLOAT_CMP_PRECISION { 1.0 / w } else { 1.0 };
        
        Vec2 {
            x: (m.m[0] * x + m.m[4] * y + m.m[12]) * inv_w,
            y: (m.m[1] * x + m.m[5] * y + m.m[13]) * inv_w,
        }
    }

    /// 绕原点旋转向量（按角度）
    pub fn rotate_by_angle_rad(&self, angle: f32) -> Vec2 {
        let s = angle.sin();
        let c = angle.cos();
        Vec2 {
            x: c * self.x - s * self.y,
            y: s * self.x + c * self.y,
        }
    }

    /// 用另一个向量旋转（复数乘法）
    pub fn rotate_vec(&self, other: &Vec2) -> Vec2 {
        Vec2 {
            x: self.x * other.x - self.y * other.y,
            y: self.x * other.y + self.y * other.x,
        }
    }

    pub fn unrotate(&self, other: &Vec2) -> Vec2 {
        Vec2 {
            x: self.x * other.x + self.y * other.y,
            y: self.y * other.x - self.x * other.y,
        }
    }

    pub fn lerp(&self, other: &Vec2, alpha: f32) -> Vec2 {
        Vec2 {
            x: self.x * (1.0 - alpha) + other.x * alpha,
            y: self.y * (1.0 - alpha) + other.y * alpha,
        }
    }

    pub fn rotate_by_angle(&self, pivot: &Vec2, angle: f32) -> Vec2 {
        let sin_angle = angle.sin();
        let cos_angle = angle.cos();

        let dx = self.x - pivot.x;
        let dy = self.y - pivot.y;

        Vec2 {
            x: cos_angle * dx - sin_angle * dy + pivot.x,
            y: sin_angle * dx + cos_angle * dy + pivot.y,
        }
    }

    pub fn for_angle(a: f32) -> Vec2 {
        Vec2 {
            x: a.cos(),
            y: a.sin(),
        }
    }

    pub fn is_line_intersect(
        a: &Vec2,
        b: &Vec2,
        c: &Vec2,
        d: &Vec2,
        s: &mut Option<f32>,
        t: &mut Option<f32>,
    ) -> bool {
        let denominator = (a.x - b.x) * (c.y - d.y) - (a.y - b.y) * (c.x - d.x);

        if denominator == 0.0 {
            return false;
        }

        let s_val = ((a.x - c.x) * (c.y - d.y) - (a.y - c.y) * (c.x - d.x)) / denominator;
        let t_val = -((a.x - b.x) * (a.y - c.y) - (a.y - b.y) * (a.x - c.x)) / denominator;

        *s = Some(s_val);
        *t = Some(t_val);

        true
    }

    pub fn is_line_overlap(a: &Vec2, b: &Vec2, c: &Vec2, d: &Vec2) -> bool {
        let (a_slope, a_intercept) = Self::line_equation(a, b);
        let (b_slope, b_intercept) = Self::line_equation(c, d);

        if a_slope == b_slope && a_intercept == b_intercept {
            true
        } else {
            false
        }
    }

    pub fn is_line_parallel(a: &Vec2, b: &Vec2, c: &Vec2, d: &Vec2) -> bool {
        let (a_slope, _) = Self::line_equation(a, b);
        let (b_slope, _) = Self::line_equation(c, d);

        a_slope == b_slope
    }

    pub fn is_segment_overlap(
        a: &Vec2,
        b: &Vec2,
        c: &Vec2,
        d: &Vec2,
        s: &mut Option<Vec2>,
        e: &mut Option<Vec2>,
    ) -> bool {
        if !Self::is_line_overlap(a, b, c, d) {
            return false;
        }

        let mut points = vec![a, b, c, d];
        points.sort_by(|p1, p2| p1.x.partial_cmp(&p2.x).unwrap());

        let start = points.first();
        let end = points.last();

        if start.is_none() || end.is_none() {
            return false;
        }

        if (points[0].x - points[3].x).abs() < FLOAT_CMP_PRECISION {
            points.sort_by(|p1, p2| p1.y.partial_cmp(&p2.y).unwrap());
            *s = Some(*points[0]);
            *e = Some(*points[3]);
        } else {
            *s = Some(*points[0]);
            *e = Some(*points[3]);
        }

        true
    }

    pub fn is_segment_intersect(a: &Vec2, b: &Vec2, c: &Vec2, d: &Vec2) -> bool {
        let mut s = None;
        let mut t = None;

        if !Self::is_line_intersect(a, b, c, d, &mut s, &mut t) {
            return false;
        }

        let s_val = s.unwrap();
        let t_val = t.unwrap();

        s_val >= 0.0 && s_val <= 1.0 && t_val >= 0.0 && t_val <= 1.0
    }

    pub fn get_intersect_point(a: &Vec2, b: &Vec2, c: &Vec2, d: &Vec2) -> Vec2 {
        let (a_slope, a_intercept) = Self::line_equation(a, b);
        let (b_slope, b_intercept) = Self::line_equation(c, d);

        let x = (b_intercept - a_intercept) / (a_slope - b_slope);
        let y = a_slope * x + a_intercept;

        Vec2::new(x, y)
    }

    fn line_equation(a: &Vec2, b: &Vec2) -> (f32, f32) {
        if (b.x - a.x).abs() < FLOAT_CMP_PRECISION {
            (f32::INFINITY, a.x)
        } else {
            let slope = (b.y - a.y) / (b.x - a.x);
            let intercept = a.y - slope * a.x;
            (slope, intercept)
        }
    }
}

impl Default for Vec2 {
    fn default() -> Self {
        Vec2::ZERO
    }
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: f32) -> Vec2 {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl Div<f32> for Vec2 {
    type Output = Vec2;

    fn div(self, rhs: f32) -> Vec2 {
        Vec2 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Neg for Vec2 {
    type Output = Vec2;

    fn neg(self) -> Vec2 {
        Vec2 {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl PartialOrd for Vec2 {
    fn partial_cmp(&self, other: &Vec2) -> Option<std::cmp::Ordering> {
        if self.x < other.x && self.y < other.y {
            Some(std::cmp::Ordering::Less)
        } else if self.x > other.x && self.y > other.y {
            Some(std::cmp::Ordering::Greater)
        } else if self.x == other.x && self.y == other.y {
            Some(std::cmp::Ordering::Equal)
        } else {
            None
        }
    }
}

pub type Point = Vec2;

impl std::fmt::Display for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vec2({:.2}, {:.2})", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    const EPSILON: f32 = 0.0001;

    fn assert_float_eq(a: f32, b: f32, epsilon: f32) {
        assert!(
            (a - b).abs() < epsilon,
            "Float values not equal: {} != {}",
            a,
            b
        );
    }

    fn assert_vec2_eq(a: &Vec2, b: &Vec2, epsilon: f32) {
        assert_float_eq(a.x, b.x, epsilon);
        assert_float_eq(a.y, b.y, epsilon);
    }

    #[test]
    fn test_constants() {
        assert_eq!(Vec2::ZERO, Vec2::new(0.0, 0.0));
        assert_eq!(Vec2::ONE, Vec2::new(1.0, 1.0));
        assert_eq!(Vec2::UNIT_X, Vec2::new(1.0, 0.0));
        assert_eq!(Vec2::UNIT_Y, Vec2::new(0.0, 1.0));
        assert_eq!(Vec2::ANCHOR_MIDDLE, Vec2::new(0.5, 0.5));
    }

    #[test]
    fn test_new() {
        let v = Vec2::new(1.0, 2.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
    }

    #[test]
    fn test_from_array() {
        let arr = [1.0, 2.0];
        let v = Vec2::from_array(&arr);
        assert_eq!(v, Vec2::new(1.0, 2.0));

        let short_arr = [1.0];
        let v2 = Vec2::from_array(&short_arr);
        assert_eq!(v2, Vec2::ZERO);
    }

    #[test]
    fn test_from_points() {
        let p1 = Vec2::new(1.0, 2.0);
        let p2 = Vec2::new(4.0, 6.0);
        let v = Vec2::from_points(&p1, &p2);
        assert_eq!(v, Vec2::new(3.0, 4.0));
    }

    #[test]
    fn test_is_zero() {
        assert!(Vec2::ZERO.is_zero());
        assert!(!Vec2::ONE.is_zero());
    }

    #[test]
    fn test_is_one() {
        assert!(Vec2::ONE.is_one());
        assert!(!Vec2::ZERO.is_one());
    }

    #[test]
    fn test_length() {
        let v = Vec2::new(3.0, 4.0);
        assert_eq!(v.length(), 5.0);
    }

    #[test]
    fn test_length_squared() {
        let v = Vec2::new(3.0, 4.0);
        assert_eq!(v.length_squared(), 25.0);
    }

    #[test]
    fn test_normalize() {
        let mut v = Vec2::new(3.0, 4.0);
        v.normalize();
        assert_float_eq(v.length(), 1.0, EPSILON);
    }

    #[test]
    fn test_get_normalized() {
        let v = Vec2::new(3.0, 4.0);
        let normalized = v.get_normalized();
        assert_float_eq(normalized.length(), 1.0, EPSILON);
        assert_vec2_eq(&normalized, &Vec2::new(0.6, 0.8), EPSILON);
    }

    #[test]
    fn test_get_normalized_zero() {
        let v = Vec2::ZERO;
        let normalized = v.get_normalized();
        assert_eq!(normalized, Vec2::ZERO);
    }

    #[test]
    fn test_dot() {
        let v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(3.0, 4.0);
        assert_eq!(v1.dot(&v2), 11.0);
    }

    #[test]
    fn test_cross() {
        let v1 = Vec2::new(1.0, 0.0);
        let v2 = Vec2::new(0.0, 1.0);
        assert_eq!(v1.cross(&v2), 1.0);
    }

    #[test]
    fn test_distance() {
        let v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(4.0, 6.0);
        assert_eq!(v1.distance(&v2), 5.0);
    }

    #[test]
    fn test_distance_squared() {
        let v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(4.0, 6.0);
        assert_eq!(v1.distance_squared(&v2), 25.0);
    }

    #[test]
    fn test_angle() {
        let v1 = Vec2::UNIT_X;
        let v2 = Vec2::UNIT_Y;
        let angle = Vec2::angle(&v1, &v2);
        assert_float_eq(angle, PI / 2.0, EPSILON);
    }

    #[test]
    fn test_add() {
        let mut v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(3.0, 4.0);
        Vec2::add(&mut v1, &v2);
        assert_eq!(v1, Vec2::new(4.0, 6.0));
    }

    #[test]
    fn test_add_vecs() {
        let v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(3.0, 4.0);
        let result = Vec2::add_vecs(&v1, &v2);
        assert_eq!(result, Vec2::new(4.0, 6.0));
    }

    #[test]
    fn test_subtract() {
        let mut v1 = Vec2::new(4.0, 6.0);
        let v2 = Vec2::new(1.0, 2.0);
        v1.subtract(&v2);
        assert_eq!(v1, Vec2::new(3.0, 4.0));
    }

    #[test]
    fn test_subtract_vecs() {
        let v1 = Vec2::new(4.0, 6.0);
        let v2 = Vec2::new(1.0, 2.0);
        let result = Vec2::subtract_vecs(&v1, &v2);
        assert_eq!(result, Vec2::new(3.0, 4.0));
    }

    #[test]
    fn test_scale() {
        let mut v = Vec2::new(1.0, 2.0);
        v.scale(2.0);
        assert_eq!(v, Vec2::new(2.0, 4.0));
    }

    #[test]
    fn test_scale_vec() {
        let v = Vec2::new(2.0, 3.0);
        let scale = Vec2::new(2.0, 3.0);
        let result = v.scale_vec(&scale);
        assert_eq!(result, Vec2::new(4.0, 9.0));
    }

    #[test]
    fn test_negate() {
        let mut v = Vec2::new(1.0, 2.0);
        v.negate();
        assert_eq!(v, Vec2::new(-1.0, -2.0));
    }

    #[test]
    fn test_clamp() {
        let mut v = Vec2::new(5.0, -2.0);
        let min = Vec2::new(0.0, 0.0);
        let max = Vec2::new(3.0, 3.0);
        v.clamp(&min, &max);
        assert_eq!(v, Vec2::new(3.0, 0.0));
    }

    #[test]
    fn test_clamp_vec() {
        let v = Vec2::new(5.0, -2.0);
        let min = Vec2::new(0.0, 0.0);
        let max = Vec2::new(3.0, 3.0);
        let result = Vec2::clamp_vec(&v, &min, &max);
        assert_eq!(result, Vec2::new(3.0, 0.0));
    }

    #[test]
    fn test_lerp() {
        let v1 = Vec2::new(0.0, 0.0);
        let v2 = Vec2::new(10.0, 10.0);
        let result = v1.lerp(&v2, 0.5);
        assert_eq!(result, Vec2::new(5.0, 5.0));
    }

    #[test]
    fn test_get_angle() {
        let v = Vec2::new(1.0, 0.0);
        assert_float_eq(v.get_angle(), 0.0, EPSILON);

        let v2 = Vec2::new(0.0, 1.0);
        assert_float_eq(v2.get_angle(), PI / 2.0, EPSILON);
    }

    #[test]
    fn test_get_perp() {
        let v = Vec2::new(1.0, 0.0);
        let perp = v.get_perp();
        assert_eq!(perp, Vec2::new(0.0, 1.0));
    }

    #[test]
    fn test_get_rperp() {
        let v = Vec2::new(1.0, 0.0);
        let rperp = v.get_rperp();
        assert_eq!(rperp, Vec2::new(0.0, -1.0));
    }

    #[test]
    fn test_project() {
        let v = Vec2::new(1.0, 1.0);
        let onto = Vec2::new(1.0, 0.0);
        let proj = v.project(&onto);
        assert_vec2_eq(&proj, &Vec2::new(1.0, 0.0), EPSILON);
    }

    #[test]
    fn test_reject() {
        let v = Vec2::new(1.0, 1.0);
        let onto = Vec2::new(1.0, 0.0);
        let rej = v.reject(&onto);
        assert_vec2_eq(&rej, &Vec2::new(0.0, 1.0), EPSILON);
    }

    #[test]
    fn test_get_midpoint() {
        let v1 = Vec2::new(0.0, 0.0);
        let v2 = Vec2::new(2.0, 4.0);
        let mid = v1.get_midpoint(&v2);
        assert_eq!(mid, Vec2::new(1.0, 2.0));
    }

    #[test]
    fn test_rotate() {
        let mut v = Vec2::new(1.0, 0.0);
        let center = Vec2::new(0.0, 0.0);
        v.rotate(&center, PI / 2.0);
        assert_float_eq(v.x, 0.0, EPSILON);
        assert_float_eq(v.y, 1.0, EPSILON);
    }

    #[test]
    fn test_rotate_by_angle_rad() {
        let v = Vec2::new(1.0, 0.0);
        let rotated = v.rotate_by_angle_rad(PI / 2.0);
        assert_float_eq(rotated.x, 0.0, EPSILON);
        assert_float_eq(rotated.y, 1.0, EPSILON);
    }

    #[test]
    fn test_rotate_vec() {
        let v1 = Vec2::new(1.0, 0.0);
        let v2 = Vec2::new(0.0, 1.0);
        let result = v1.rotate_vec(&v2);
        assert_float_eq(result.x, 0.0, EPSILON);
        assert_float_eq(result.y, 1.0, EPSILON);
    }

    #[test]
    fn test_unrotate() {
        let v1 = Vec2::new(0.0, 1.0);
        let v2 = Vec2::new(0.0, 1.0);
        let result = v1.unrotate(&v2);
        assert_float_eq(result.x, 1.0, EPSILON);
        assert_float_eq(result.y, 0.0, EPSILON);
    }

    #[test]
    fn test_for_angle() {
        let v = Vec2::for_angle(PI / 2.0);
        assert_float_eq(v.x, 0.0, EPSILON);
        assert_float_eq(v.y, 1.0, EPSILON);
    }

    #[test]
    fn test_equals() {
        let v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(1.000001, 2.000001);
        assert!(v1.equals(&v2));
    }

    #[test]
    fn test_fuzzy_equals() {
        let v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(1.1, 2.1);
        assert!(v1.fuzzy_equals(&v2, 0.2));
        assert!(!v1.fuzzy_equals(&v2, 0.05));
    }

    #[test]
    fn test_smooth() {
        let mut v = Vec2::new(0.0, 0.0);
        let target = Vec2::new(10.0, 10.0);
        v.smooth(&target, 1.0, 1.0);
        // Should move towards target
        assert!(v.x > 0.0);
        assert!(v.y > 0.0);
    }

    #[test]
    fn test_add_operator() {
        let v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(3.0, 4.0);
        let result = v1 + v2;
        assert_eq!(result, Vec2::new(4.0, 6.0));
    }

    #[test]
    fn test_sub_operator() {
        let v1 = Vec2::new(4.0, 6.0);
        let v2 = Vec2::new(1.0, 2.0);
        let result = v1 - v2;
        assert_eq!(result, Vec2::new(3.0, 4.0));
    }

    #[test]
    fn test_mul_scalar_operator() {
        let v = Vec2::new(1.0, 2.0);
        let result = v * 2.0;
        assert_eq!(result, Vec2::new(2.0, 4.0));
    }

    #[test]
    fn test_div_scalar_operator() {
        let v = Vec2::new(2.0, 4.0);
        let result = v / 2.0;
        assert_eq!(result, Vec2::new(1.0, 2.0));
    }

    #[test]
    fn test_neg_operator() {
        let v = Vec2::new(1.0, 2.0);
        let result = -v;
        assert_eq!(result, Vec2::new(-1.0, -2.0));
    }

    #[test]
    fn test_partial_ord() {
        let v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(2.0, 3.0);
        let v3 = Vec2::new(1.0, 2.0);

        assert_eq!(v1.partial_cmp(&v2), Some(std::cmp::Ordering::Less));
        assert_eq!(v2.partial_cmp(&v1), Some(std::cmp::Ordering::Greater));
        assert_eq!(v1.partial_cmp(&v3), Some(std::cmp::Ordering::Equal));

        // Incomparable case
        let v4 = Vec2::new(2.0, 1.0);
        assert_eq!(v1.partial_cmp(&v4), None);
    }

    #[test]
    fn test_default() {
        let v: Vec2 = Default::default();
        assert_eq!(v, Vec2::ZERO);
    }

    #[test]
    fn test_display() {
        let v = Vec2::new(1.0, 2.0);
        assert_eq!(format!("{}", v), "Vec2(1.00, 2.00)");
    }
}
