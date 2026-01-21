use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};

const FLOAT_CMP_PRECISION: f32 = 0.00001;

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
