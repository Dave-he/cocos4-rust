use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};

const FLOAT_CMP_PRECISION: f32 = 0.00001;

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
            normalized * max_step
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
