use super::Vec2;
use super::FLOAT_CMP_PRECISION;
use std::ops::{Add, Div, Mul, MulAssign, Sub};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const ZERO: Size = Size {
        width: 0.0,
        height: 0.0,
    };

    pub fn new(width: f32, height: f32) -> Self {
        Size { width, height }
    }

    pub fn from_vec2(v: &Vec2) -> Self {
        Size {
            width: v.x,
            height: v.y,
        }
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn equals(&self, target: &Size) -> bool {
        (self.width - target.width).abs() < FLOAT_CMP_PRECISION
            && (self.height - target.height).abs() < FLOAT_CMP_PRECISION
    }

    pub fn to_vec2(&self) -> Vec2 {
        Vec2 {
            x: self.width,
            y: self.height,
        }
    }
}

impl Default for Size {
    fn default() -> Self {
        Size::ZERO
    }
}

impl Add for Size {
    type Output = Size;

    fn add(self, rhs: Size) -> Size {
        Size {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl Sub for Size {
    type Output = Size;

    fn sub(self, rhs: Size) -> Size {
        Size {
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}

impl Mul<f32> for Size {
    type Output = Size;

    fn mul(self, rhs: f32) -> Size {
        Size {
            width: self.width * rhs,
            height: self.height * rhs,
        }
    }
}

impl MulAssign<f32> for Size {
    fn mul_assign(&mut self, rhs: f32) {
        self.width *= rhs;
        self.height *= rhs;
    }
}

impl Div<f32> for Size {
    type Output = Size;

    fn div(self, rhs: f32) -> Size {
        Size {
            width: self.width / rhs,
            height: self.height / rhs,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const ZERO: Rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };

    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_vec_size(pos: &Vec2, dimension: &Size) -> Self {
        Rect {
            x: pos.x,
            y: pos.y,
            width: dimension.width,
            height: dimension.height,
        }
    }

    pub fn set_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
    }

    pub fn get_min_x(&self) -> f32 {
        self.x
    }

    pub fn get_mid_x(&self) -> f32 {
        self.x + self.width * 0.5
    }

    pub fn get_max_x(&self) -> f32 {
        self.x + self.width
    }

    pub fn get_min_y(&self) -> f32 {
        self.y
    }

    pub fn get_mid_y(&self) -> f32 {
        self.y + self.height * 0.5
    }

    pub fn get_max_y(&self) -> f32 {
        self.y + self.height
    }

    pub fn equals(&self, rect: &Rect) -> bool {
        (self.x - rect.x).abs() < FLOAT_CMP_PRECISION
            && (self.y - rect.y).abs() < FLOAT_CMP_PRECISION
            && (self.width - rect.width).abs() < FLOAT_CMP_PRECISION
            && (self.height - rect.height).abs() < FLOAT_CMP_PRECISION
    }

    pub fn contains_point(&self, point: &Vec2) -> bool {
        point.x >= self.x
            && point.x <= self.get_max_x()
            && point.y >= self.y
            && point.y <= self.get_max_y()
    }

    pub fn intersects_rect(&self, rect: &Rect) -> bool {
        !(self.get_max_x() < rect.x
            || rect.get_max_x() < self.x
            || self.get_max_y() < rect.y
            || rect.get_max_y() < self.y)
    }

    pub fn intersects_circle(&self, center: &Vec2, radius: f32) -> bool {
        let closest_x = center.x.max(self.x).min(self.get_max_x());
        let closest_y = center.y.max(self.y).min(self.get_max_y());

        let distance_x = center.x - closest_x;
        let distance_y = center.y - closest_y;

        (distance_x * distance_x + distance_y * distance_y) <= radius * radius
    }

    pub fn union_with_rect(&self, rect: &Rect) -> Rect {
        let min_x = self.x.min(rect.x);
        let min_y = self.y.min(rect.y);
        let max_x = self.get_max_x().max(rect.get_max_x());
        let max_y = self.get_max_y().max(rect.get_max_y());

        Rect {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }

    pub fn merge(&mut self, rect: &Rect) {
        *self = self.union_with_rect(rect);
    }
}

impl Default for Rect {
    fn default() -> Self {
        Rect::ZERO
    }
}
