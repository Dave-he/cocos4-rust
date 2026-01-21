use super::Vec3;
use super::Vec4;

const MATRIX4_SIZE: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4 {
    pub m: [f32; MATRIX4_SIZE],
}

impl Mat4 {
    pub const ZERO: Mat4 = Mat4 {
        m: [0.0; MATRIX4_SIZE],
    };
    pub const IDENTITY: Mat4 = Mat4 {
        m: [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ],
    };

    pub fn new(
        m00: f32,
        m01: f32,
        m02: f32,
        m03: f32,
        m04: f32,
        m05: f32,
        m06: f32,
        m07: f32,
        m08: f32,
        m09: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m13: f32,
        m14: f32,
        m15: f32,
    ) -> Self {
        Mat4 {
            m: [
                m00, m01, m02, m03, m04, m05, m06, m07, m08, m09, m10, m11, m12, m13, m14, m15,
            ],
        }
    }

    pub fn from_array(mat: &[f32]) -> Self {
        if mat.len() >= MATRIX4_SIZE {
            let mut m = [0.0; MATRIX4_SIZE];
            m.copy_from_slice(&mat[..MATRIX4_SIZE]);
            Mat4 { m }
        } else {
            Mat4::ZERO
        }
    }

    pub fn from_mat4(copy: &Mat4) -> Self {
        *copy
    }

    pub fn set(
        &mut self,
        m00: f32,
        m01: f32,
        m02: f32,
        m03: f32,
        m04: f32,
        m05: f32,
        m06: f32,
        m07: f32,
        m08: f32,
        m09: f32,
        m10: f32,
        m11: f32,
        m12: f32,
        m13: f32,
        m14: f32,
        m15: f32,
    ) {
        self.m[0] = m00;
        self.m[1] = m01;
        self.m[2] = m02;
        self.m[3] = m03;
        self.m[4] = m04;
        self.m[5] = m05;
        self.m[6] = m06;
        self.m[7] = m07;
        self.m[8] = m08;
        self.m[9] = m09;
        self.m[10] = m10;
        self.m[11] = m11;
        self.m[12] = m12;
        self.m[13] = m13;
        self.m[14] = m14;
        self.m[15] = m15;
    }

    pub fn set_from_array(&mut self, mat: &[f32]) {
        if mat.len() >= MATRIX4_SIZE {
            self.m.copy_from_slice(&mat[..MATRIX4_SIZE]);
        }
    }

    pub fn identity(mat: &mut Mat4) {
        mat.m.copy_from_slice(&Mat4::IDENTITY.m);
    }

    pub fn transpose(&mut self) {
        let mut result = *self;
        result.m.swap(1, 4);
        result.m.swap(2, 8);
        result.m.swap(3, 12);
        result.m.swap(4, 1);
        result.m.swap(5, 9);
        result.m.swap(6, 13);
        result.m.swap(7, 14);
        result.m.swap(8, 10);
        result.m.swap(9, 15);
        *self = result;
    }

    pub fn multiply(a: &Mat4, b: &Mat4, out: &mut Mat4) {
        for i in 0..4 {
            for j in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += a.m[i * 4 + k] * b.m[j + k];
                }
                out.m[i + j * 4] = sum;
            }
        }
    }

    pub fn multiply_vec4(&self, vec: &Vec4) -> Vec4 {
        Vec4::new(
            self.m[0] * vec.x + self.m[1] * vec.y + self.m[2] * vec.z + self.m[3] * vec.w,
            self.m[4] * vec.x + self.m[5] * vec.y + self.m[6] * vec.z + self.m[7] * vec.w,
            self.m[8] * vec.x + self.m[9] * vec.y + self.m[10] * vec.z + self.m[11] * vec.w,
            self.m[12] * vec.x + self.m[13] * vec.y + self.m[14] * vec.z + self.m[15] * vec.w,
        )
    }

    pub fn translate(&mut self, vec: &Vec3) {
        self.m[12] = vec.x;
        self.m[13] = vec.y;
        self.m[14] = vec.z;
    }

    pub fn from_translation(translation: &Vec3) -> Mat4 {
        let mut result = Mat4::IDENTITY;
        result.translate(translation);
        result
    }
}

impl Default for Mat4 {
    fn default() -> Self {
        Mat4::ZERO
    }
}
