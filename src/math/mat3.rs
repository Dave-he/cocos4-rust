use super::Vec3;

const MATRIX3_SIZE: usize = 9;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat3 {
    pub m: [f32; MATRIX3_SIZE],
}

impl Mat3 {
    pub const ZERO: Mat3 = Mat3 {
        m: [0.0; MATRIX3_SIZE],
    };
    pub const IDENTITY: Mat3 = Mat3 {
        m: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
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
    ) -> Self {
        Mat3 {
            m: [m00, m01, m02, m03, m04, m05, m06, m07, m08],
        }
    }

    pub fn from_array(mat: &[f32]) -> Self {
        if mat.len() >= MATRIX3_SIZE {
            let mut m = [0.0; MATRIX3_SIZE];
            m.copy_from_slice(&mat[..MATRIX3_SIZE]);
            Mat3 { m }
        } else {
            Mat3::ZERO
        }
    }

    pub fn from_mat3(copy: &Mat3) -> Self {
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
    }

    pub fn set_from_array(&mut self, mat: &[f32]) {
        if mat.len() >= MATRIX3_SIZE {
            self.m.copy_from_slice(&mat[..MATRIX3_SIZE]);
        }
    }

    pub fn set_from_mat3(&mut self, mat: &Mat3) {
        self.m.copy_from_slice(&mat.m);
    }

    pub fn identity(mat: &mut Mat3) {
        mat.m.copy_from_slice(&Mat3::IDENTITY.m);
    }

    pub fn transpose(&mut self) {
        let m00 = self.m[0];
        let m01 = self.m[1];
        let m02 = self.m[2];
        let m10 = self.m[3];
        let m11 = self.m[4];
        let m12 = self.m[5];
        let m20 = self.m[6];
        let m21 = self.m[7];
        let m22 = self.m[8];

        self.m[0] = m00;
        self.m[1] = m10;
        self.m[2] = m20;
        self.m[3] = m01;
        self.m[4] = m11;
        self.m[5] = m21;
        self.m[6] = m02;
        self.m[7] = m12;
        self.m[8] = m22;
    }

    pub fn transpose_mat(mat: &Mat3, out: &mut Mat3) {
        out.m[0] = mat.m[0];
        out.m[1] = mat.m[3];
        out.m[2] = mat.m[6];
        out.m[3] = mat.m[1];
        out.m[4] = mat.m[4];
        out.m[5] = mat.m[7];
        out.m[6] = mat.m[2];
        out.m[7] = mat.m[5];
        out.m[8] = mat.m[8];
    }

    pub fn inverse(&mut self) {
        let m00 = self.m[0];
        let m01 = self.m[1];
        let m02 = self.m[2];
        let m10 = self.m[3];
        let m11 = self.m[4];
        let m12 = self.m[5];
        let m20 = self.m[6];
        let m21 = self.m[7];
        let m22 = self.m[8];

        let det = self.determinant();
        if det.abs() < 0.000001 {
            return;
        }

        let inv_det = 1.0 / det;

        let cof00 = m11 * m22 - m12 * m21;
        let cof01 = m02 * m21 - m22 * m20;
        let cof02 = m12 * m20 - m10 * m22;
        let cof10 = m02 * m12 - m22 * m10;
        let cof11 = m00 * m22 - m02 * m20;
        let cof12 = m10 * m20 - m00 * m12;
        let cof20 = m10 * m21 - m11 * m20;
        let cof21 = m01 * m20 - m21 * m00;
        let cof22 = m00 * m11 - m01 * m10;

        self.m[0] = cof00 * inv_det;
        self.m[1] = cof10 * inv_det;
        self.m[2] = cof20 * inv_det;
        self.m[3] = cof01 * inv_det;
        self.m[4] = cof11 * inv_det;
        self.m[5] = cof21 * inv_det;
        self.m[6] = cof02 * inv_det;
        self.m[7] = cof12 * inv_det;
        self.m[8] = cof22 * inv_det;
    }

    pub fn determinant(&self) -> f32 {
        let m00 = self.m[0];
        let m01 = self.m[1];
        let m02 = self.m[2];
        let m10 = self.m[3];
        let m11 = self.m[4];
        let m12 = self.m[5];
        let m20 = self.m[6];
        let m21 = self.m[7];
        let m22 = self.m[8];

        m00 * (m11 * m22 - m12 * m21) - m01 * (m10 * m22 - m12 * m20)
            + m02 * (m10 * m21 - m11 * m20)
    }

    pub fn multiply(a: &Mat3, b: &Mat3, out: &mut Mat3) {
        let a00 = a.m[0];
        let a01 = a.m[1];
        let a02 = a.m[2];
        let a10 = a.m[3];
        let a11 = a.m[4];
        let a12 = a.m[5];
        let a20 = a.m[6];
        let a21 = a.m[7];
        let a22 = a.m[8];

        let b00 = b.m[0];
        let b01 = b.m[1];
        let b02 = b.m[2];
        let b10 = b.m[3];
        let b11 = b.m[4];
        let b12 = b.m[5];
        let b20 = b.m[6];
        let b21 = b.m[7];
        let b22 = b.m[8];

        out.m[0] = a00 * b00 + a01 * b10 + a02 * b20;
        out.m[1] = a00 * b01 + a01 * b11 + a02 * b21;
        out.m[2] = a00 * b02 + a01 * b12 + a02 * b22;
        out.m[3] = a10 * b00 + a11 * b10 + a12 * b20;
        out.m[4] = a10 * b01 + a11 * b11 + a12 * b21;
        out.m[5] = a10 * b02 + a11 * b12 + a12 * b22;
        out.m[6] = a20 * b00 + a21 * b10 + a22 * b20;
        out.m[7] = a20 * b01 + a21 * b11 + a22 * b21;
        out.m[8] = a20 * b02 + a21 * b12 + a22 * b22;
    }

    pub fn multiply_vec3(&self, vec: &Vec3) -> Vec3 {
        Vec3::new(
            self.m[0] * vec.x + self.m[1] * vec.y + self.m[2] * vec.z,
            self.m[3] * vec.x + self.m[4] * vec.y + self.m[5] * vec.z,
            self.m[6] * vec.x + self.m[7] * vec.y + self.m[8] * vec.z,
        )
    }

    pub fn add(a: &Mat3, b: &Mat3, out: &mut Mat3) {
        for i in 0..MATRIX3_SIZE {
            out.m[i] = a.m[i] + b.m[i];
        }
    }

    pub fn subtract(a: &Mat3, b: &Mat3, out: &mut Mat3) {
        for i in 0..MATRIX3_SIZE {
            out.m[i] = a.m[i] - b.m[i];
        }
    }

    pub fn approx_equals(&self, v: &Mat3, precision: f32) -> bool {
        for i in 0..MATRIX3_SIZE {
            if (self.m[i] - v.m[i]).abs() >= precision {
                return false;
            }
        }
        true
    }
}

impl Default for Mat3 {
    fn default() -> Self {
        Mat3::ZERO
    }
}
