use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};

use super::Mat4;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub const ZERO: Vec4 = Vec4 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };
    pub const ONE: Vec4 = Vec4 {
        x: 1.0,
        y: 1.0,
        z: 1.0,
        w: 1.0,
    };
    pub const UNIT_X: Vec4 = Vec4 {
        x: 1.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };
    pub const UNIT_Y: Vec4 = Vec4 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
        w: 0.0,
    };
    pub const UNIT_Z: Vec4 = Vec4 {
        x: 0.0,
        y: 0.0,
        z: 1.0,
        w: 0.0,
    };
    pub const UNIT_W: Vec4 = Vec4 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Vec4 { x, y, z, w }
    }

    pub fn from_array(array: &[f32]) -> Self {
        if array.len() >= 4 {
            Vec4 {
                x: array[0],
                y: array[1],
                z: array[2],
                w: array[3],
            }
        } else {
            Vec4::ZERO
        }
    }

    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0 && self.z == 0.0 && self.w == 0.0
    }

    pub fn is_one(&self) -> bool {
        self.x == 1.0 && self.y == 1.0 && self.z == 1.0 && self.w == 1.0
    }

    pub fn add(&mut self, v: &Vec4) {
        self.x += v.x;
        self.y += v.y;
        self.z += v.z;
        self.w += v.w;
    }

    pub fn add_vecs(v1: &Vec4, v2: &Vec4) -> Vec4 {
        Vec4 {
            x: v1.x + v2.x,
            y: v1.y + v2.y,
            z: v1.z + v2.z,
            w: v1.w + v2.w,
        }
    }

    pub fn clamp(&mut self, min: &Vec4, max: &Vec4) {
        self.x = self.x.max(min.x).min(max.x);
        self.y = self.y.max(min.y).min(max.y);
        self.z = self.z.max(min.z).min(max.z);
        self.w = self.w.max(min.w).min(max.w);
    }

    pub fn clamp_vec(v: &Vec4, min: &Vec4, max: &Vec4) -> Vec4 {
        Vec4 {
            x: v.x.max(min.x).min(max.x),
            y: v.y.max(min.y).min(max.y),
            z: v.z.max(min.z).min(max.z),
            w: v.w.max(min.w).min(max.w),
        }
    }

    pub fn distance(&self, v: &Vec4) -> f32 {
        let dx = self.x - v.x;
        let dy = self.y - v.y;
        let dz = self.z - v.z;
        let dw = self.w - v.w;
        (dx * dx + dy * dy + dz * dz + dw * dw).sqrt()
    }

    pub fn distance_squared(&self, v: &Vec4) -> f32 {
        let dx = self.x - v.x;
        let dy = self.y - v.y;
        let dz = self.z - v.z;
        let dw = self.w - v.w;
        dx * dx + dy * dy + dz * dz + dw * dw
    }

    pub fn dot(&self, v: &Vec4) -> f32 {
        self.x * v.x + self.y * v.y + self.z * v.z + self.w * v.w
    }

    pub fn dot_vecs(v1: &Vec4, v2: &Vec4) -> f32 {
        v1.x * v2.x + v1.y * v2.y + v1.z * v2.z + v1.w * v2.w
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    pub fn negate(&mut self) {
        self.x = -self.x;
        self.y = -self.y;
        self.z = -self.z;
        self.w = -self.w;
    }

    pub fn normalize(&mut self) {
        let len = self.length();
        if len > 0.0 {
            self.x /= len;
            self.y /= len;
            self.z /= len;
            self.w /= len;
        }
    }

    pub fn get_normalized(&self) -> Vec4 {
        let len = self.length();
        if len > 0.0 {
            Vec4 {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
                w: self.w / len,
            }
        } else {
            *self
        }
    }

    pub fn scale(&mut self, scalar: f32) {
        self.x *= scalar;
        self.y *= scalar;
        self.z *= scalar;
        self.w *= scalar;
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

    pub fn set_from_vec(&mut self, v: &Vec4) {
        self.x = v.x;
        self.y = v.y;
        self.z = v.z;
        self.w = v.w;
    }

    pub fn set_zero(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
        self.z = 0.0;
        self.w = 0.0;
    }

    pub fn subtract(&mut self, v: &Vec4) {
        self.x -= v.x;
        self.y -= v.y;
        self.z -= v.z;
        self.w -= v.w;
    }

    pub fn subtract_vecs(v1: &Vec4, v2: &Vec4) -> Vec4 {
        Vec4 {
            x: v1.x - v2.x,
            y: v1.y - v2.y,
            z: v1.z - v2.z,
            w: v1.w - v2.w,
        }
    }

    pub fn lerp(&self, target: &Vec4, alpha: f32) -> Vec4 {
        Vec4 {
            x: self.x * (1.0 - alpha) + target.x * alpha,
            y: self.y * (1.0 - alpha) + target.y * alpha,
            z: self.z * (1.0 - alpha) + target.z * alpha,
            w: self.w * (1.0 - alpha) + target.w * alpha,
        }
    }

    /// 用 4x4 矩阵变换向量
    pub fn transform_mat4(&self, m: &Mat4) -> Vec4 {
        Vec4 {
            x: m.m[0] * self.x + m.m[4] * self.y + m.m[8] * self.z + m.m[12] * self.w,
            y: m.m[1] * self.x + m.m[5] * self.y + m.m[9] * self.z + m.m[13] * self.w,
            z: m.m[2] * self.x + m.m[6] * self.y + m.m[10] * self.z + m.m[14] * self.w,
            w: m.m[3] * self.x + m.m[7] * self.y + m.m[11] * self.z + m.m[15] * self.w,
        }
    }

    /// 计算两个向量之间的夹角
    pub fn angle(a: &Vec4, b: &Vec4) -> f32 {
        let dx = a.y * b.z - a.z * b.y;
        let dy = a.z * b.x - a.x * b.z;
        let dz = a.x * b.y - a.y * b.x;
        
        let dot_val = a.x * b.x + a.y * b.y + a.z * b.z;
        (dx * dx + dy * dy + dz * dz).sqrt().atan2(dot_val)
    }

    /// 计算向量在另一个向量上的投影
    pub fn project(&self, other: &Vec4) -> Vec4 {
        let dot_self = self.dot(other);
        let dot_other = other.dot(other);
        *other * (dot_self / dot_other)
    }

    /// 计算向量在另一个向量上的拒绝
    pub fn reject(&self, other: &Vec4) -> Vec4 {
        *self - self.project(other)
    }

    /// 获取两个向量之间的夹角
    pub fn get_angle(&self, other: &Vec4) -> f32 {
        Self::angle(self, other)
    }
}

impl Default for Vec4 {
    fn default() -> Self {
        Vec4::ZERO
    }
}

impl Add for Vec4 {
    type Output = Vec4;

    fn add(self, rhs: Vec4) -> Vec4 {
        Vec4 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}

impl AddAssign for Vec4 {
    fn add_assign(&mut self, rhs: Vec4) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self.w += rhs.w;
    }
}

impl Sub for Vec4 {
    type Output = Vec4;

    fn sub(self, rhs: Vec4) -> Vec4 {
        Vec4 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: self.w - rhs.w,
        }
    }
}

impl SubAssign for Vec4 {
    fn sub_assign(&mut self, rhs: Vec4) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
        self.w -= rhs.w;
    }
}

impl Mul<f32> for Vec4 {
    type Output = Vec4;

    fn mul(self, rhs: f32) -> Vec4 {
        Vec4 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            w: self.w * rhs,
        }
    }
}

impl MulAssign<f32> for Vec4 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
        self.w *= rhs;
    }
}

impl Mul<Vec4> for f32 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Vec4 {
        Vec4 {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
            w: self * rhs.w,
        }
    }
}

impl Div<f32> for Vec4 {
    type Output = Vec4;

    fn div(self, rhs: f32) -> Vec4 {
        Vec4 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
            w: self.w / rhs,
        }
    }
}

impl Neg for Vec4 {
    type Output = Vec4;

    fn neg(self) -> Vec4 {
        Vec4 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}
