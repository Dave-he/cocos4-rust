use super::{Quaternion, Vec3, Vec4};

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

    pub fn zero(mat: &mut Mat4) {
        mat.m.copy_from_slice(&Mat4::ZERO.m);
    }

    pub fn transpose(&mut self) {
        let a01 = self.m[1];
        let a02 = self.m[2];
        let a03 = self.m[3];
        let a12 = self.m[6];
        let a13 = self.m[7];
        let a23 = self.m[11];

        self.m[1] = self.m[4];
        self.m[2] = self.m[8];
        self.m[3] = self.m[12];
        self.m[4] = a01;
        self.m[6] = self.m[9];
        self.m[7] = self.m[13];
        self.m[8] = a02;
        self.m[9] = a12;
        self.m[11] = self.m[14];
        self.m[12] = a03;
        self.m[13] = a13;
        self.m[14] = a23;
    }

    pub fn get_transposed(&self) -> Mat4 {
        let mut result = *self;
        result.transpose();
        result
    }

    pub fn multiply(a: &Mat4, b: &Mat4, out: &mut Mat4) {
        let a00 = a.m[0];
        let a01 = a.m[1];
        let a02 = a.m[2];
        let a03 = a.m[3];
        let a10 = a.m[4];
        let a11 = a.m[5];
        let a12 = a.m[6];
        let a13 = a.m[7];
        let a20 = a.m[8];
        let a21 = a.m[9];
        let a22 = a.m[10];
        let a23 = a.m[11];
        let a30 = a.m[12];
        let a31 = a.m[13];
        let a32 = a.m[14];
        let a33 = a.m[15];

        let mut b0 = b.m[0];
        let mut b1 = b.m[1];
        let mut b2 = b.m[2];
        let mut b3 = b.m[3];
        out.m[0] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
        out.m[1] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
        out.m[2] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
        out.m[3] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;

        b0 = b.m[4];
        b1 = b.m[5];
        b2 = b.m[6];
        b3 = b.m[7];
        out.m[4] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
        out.m[5] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
        out.m[6] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
        out.m[7] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;

        b0 = b.m[8];
        b1 = b.m[9];
        b2 = b.m[10];
        b3 = b.m[11];
        out.m[8] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
        out.m[9] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
        out.m[10] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
        out.m[11] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;

        b0 = b.m[12];
        b1 = b.m[13];
        b2 = b.m[14];
        b3 = b.m[15];
        out.m[12] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
        out.m[13] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
        out.m[14] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
        out.m[15] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;
    }

    pub fn multiply_mat4(a: &Mat4, b: &Mat4) -> Mat4 {
        let mut result = Mat4::ZERO;
        Self::multiply(a, b, &mut result);
        result
    }

    pub fn multiply_vec4(&self, vec: &Vec4) -> Vec4 {
        Vec4::new(
            self.m[0] * vec.x + self.m[4] * vec.y + self.m[8] * vec.z + self.m[12] * vec.w,
            self.m[1] * vec.x + self.m[5] * vec.y + self.m[9] * vec.z + self.m[13] * vec.w,
            self.m[2] * vec.x + self.m[6] * vec.y + self.m[10] * vec.z + self.m[14] * vec.w,
            self.m[3] * vec.x + self.m[7] * vec.y + self.m[11] * vec.z + self.m[15] * vec.w,
        )
    }

    pub fn translate(&mut self, vec: &Vec3) {
        self.m[12] = vec.x;
        self.m[13] = vec.y;
        self.m[14] = vec.z;
    }

    pub fn from_translation(v: &Vec3) -> Mat4 {
        Mat4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            v.x, v.y, v.z, 1.0,
        )
    }

    pub fn from_scale(v: &Vec3) -> Mat4 {
        Mat4::new(
            v.x, 0.0, 0.0, 0.0,
            0.0, v.y, 0.0, 0.0,
            0.0, 0.0, v.z, 0.0,
            0.0, 0.0, 0.0, 1.0,
        )
    }

    pub fn from_rotation(q: &Quaternion) -> Mat4 {
        Self::from_quat(q)
    }

    pub fn from_quat(q: &Quaternion) -> Mat4 {
        let x = q.x;
        let y = q.y;
        let z = q.z;
        let w = q.w;

        let x2 = x + x;
        let y2 = y + y;
        let z2 = z + z;

        let xx = x * x2;
        let yx = y * x2;
        let yy = y * y2;
        let zx = z * x2;
        let zy = z * y2;
        let zz = z * z2;
        let wx = w * x2;
        let wy = w * y2;
        let wz = w * z2;

        Mat4::new(
            1.0 - yy - zz, yx + wz, zx - wy, 0.0,
            yx - wz, 1.0 - xx - zz, zy + wx, 0.0,
            zx + wy, zy - wx, 1.0 - xx - yy, 0.0,
            0.0, 0.0, 0.0, 1.0,
        )
    }

    pub fn determinant(&self) -> f32 {
        let a00 = self.m[0];
        let a01 = self.m[1];
        let a02 = self.m[2];
        let a03 = self.m[3];
        let a10 = self.m[4];
        let a11 = self.m[5];
        let a12 = self.m[6];
        let a13 = self.m[7];
        let a20 = self.m[8];
        let a21 = self.m[9];
        let a22 = self.m[10];
        let a23 = self.m[11];
        let a30 = self.m[12];
        let a31 = self.m[13];
        let a32 = self.m[14];
        let a33 = self.m[15];

        let b00 = a00 * a11 - a01 * a10;
        let b01 = a00 * a12 - a02 * a10;
        let b02 = a00 * a13 - a03 * a10;
        let b03 = a01 * a12 - a02 * a11;
        let b04 = a01 * a13 - a03 * a11;
        let b05 = a02 * a13 - a03 * a12;
        let b06 = a20 * a31 - a21 * a30;
        let b07 = a20 * a32 - a22 * a30;
        let b08 = a20 * a33 - a23 * a30;
        let b09 = a21 * a32 - a22 * a31;
        let b10 = a21 * a33 - a23 * a31;
        let b11 = a22 * a33 - a23 * a32;

        b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06
    }

    pub fn invert(&mut self) {
        let a00 = self.m[0];
        let a01 = self.m[1];
        let a02 = self.m[2];
        let a03 = self.m[3];
        let a10 = self.m[4];
        let a11 = self.m[5];
        let a12 = self.m[6];
        let a13 = self.m[7];
        let a20 = self.m[8];
        let a21 = self.m[9];
        let a22 = self.m[10];
        let a23 = self.m[11];
        let a30 = self.m[12];
        let a31 = self.m[13];
        let a32 = self.m[14];
        let a33 = self.m[15];

        let b00 = a00 * a11 - a01 * a10;
        let b01 = a00 * a12 - a02 * a10;
        let b02 = a00 * a13 - a03 * a10;
        let b03 = a01 * a12 - a02 * a11;
        let b04 = a01 * a13 - a03 * a11;
        let b05 = a02 * a13 - a03 * a12;
        let b06 = a20 * a31 - a21 * a30;
        let b07 = a20 * a32 - a22 * a30;
        let b08 = a20 * a33 - a23 * a30;
        let b09 = a21 * a32 - a22 * a31;
        let b10 = a21 * a33 - a23 * a31;
        let b11 = a22 * a33 - a23 * a32;

        let det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;

        if det == 0.0 {
            self.m = [0.0; MATRIX4_SIZE];
            return;
        }

        let det_inv = 1.0 / det;

        self.m[0] = (a11 * b11 - a12 * b10 + a13 * b09) * det_inv;
        self.m[1] = (a02 * b10 - a01 * b11 - a03 * b09) * det_inv;
        self.m[2] = (a31 * b05 - a32 * b04 + a33 * b03) * det_inv;
        self.m[3] = (a22 * b04 - a21 * b05 - a23 * b03) * det_inv;
        self.m[4] = (a12 * b08 - a10 * b11 - a13 * b07) * det_inv;
        self.m[5] = (a00 * b11 - a02 * b08 + a03 * b07) * det_inv;
        self.m[6] = (a32 * b02 - a30 * b05 - a33 * b01) * det_inv;
        self.m[7] = (a20 * b05 - a22 * b02 + a23 * b01) * det_inv;
        self.m[8] = (a10 * b10 - a11 * b08 + a13 * b06) * det_inv;
        self.m[9] = (a01 * b08 - a00 * b10 - a03 * b06) * det_inv;
        self.m[10] = (a30 * b04 - a31 * b02 + a33 * b00) * det_inv;
        self.m[11] = (a21 * b02 - a20 * b04 - a23 * b00) * det_inv;
        self.m[12] = (a11 * b07 - a10 * b09 - a12 * b06) * det_inv;
        self.m[13] = (a00 * b09 - a01 * b07 + a02 * b06) * det_inv;
        self.m[14] = (a31 * b01 - a30 * b03 - a32 * b00) * det_inv;
        self.m[15] = (a20 * b03 - a21 * b01 + a22 * b00) * det_inv;
    }

    pub fn get_inverted(&self) -> Mat4 {
        let mut result = *self;
        result.invert();
        result
    }

    pub fn look_at(eye: &Vec3, center: &Vec3, up: &Vec3) -> Mat4 {
        let mut z0 = eye.x - center.x;
        let mut z1 = eye.y - center.y;
        let mut z2 = eye.z - center.z;

        let mut len = 1.0 / (z0 * z0 + z1 * z1 + z2 * z2).sqrt();
        z0 *= len;
        z1 *= len;
        z2 *= len;

        let mut x0 = up.y * z2 - up.z * z1;
        let mut x1 = up.z * z0 - up.x * z2;
        let mut x2 = up.x * z1 - up.y * z0;
        len = 1.0 / (x0 * x0 + x1 * x1 + x2 * x2).sqrt();
        x0 *= len;
        x1 *= len;
        x2 *= len;

        let y0 = z1 * x2 - z2 * x1;
        let y1 = z2 * x0 - z0 * x2;
        let y2 = z0 * x1 - z1 * x0;

        Mat4::new(
            x0, y0, z0, 0.0,
            x1, y1, z1, 0.0,
            x2, y2, z2, 0.0,
            -(x0 * eye.x + x1 * eye.y + x2 * eye.z),
            -(y0 * eye.x + y1 * eye.y + y2 * eye.z),
            -(z0 * eye.x + z1 * eye.y + z2 * eye.z),
            1.0,
        )
    }

    pub fn perspective(fovy: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
        let f = 1.0 / (fovy / 2.0).tan();
        let nf = 1.0 / (near - far);

        Mat4::new(
            f / aspect, 0.0, 0.0, 0.0,
            0.0, f, 0.0, 0.0,
            0.0, 0.0, (far + near) * nf, -1.0,
            0.0, 0.0, 2.0 * far * near * nf, 0.0,
        )
    }

    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4 {
        let lr = 1.0 / (left - right);
        let bt = 1.0 / (bottom - top);
        let nf = 1.0 / (near - far);

        Mat4::new(
            -2.0 * lr, 0.0, 0.0, 0.0,
            0.0, -2.0 * bt, 0.0, 0.0,
            0.0, 0.0, 2.0 * nf, 0.0,
            (left + right) * lr, (top + bottom) * bt, (far + near) * nf, 1.0,
        )
    }

    pub fn is_identity(&self) -> bool {
        self.m[0] == 1.0
            && self.m[1] == 0.0
            && self.m[2] == 0.0
            && self.m[3] == 0.0
            && self.m[4] == 0.0
            && self.m[5] == 1.0
            && self.m[6] == 0.0
            && self.m[7] == 0.0
            && self.m[8] == 0.0
            && self.m[9] == 0.0
            && self.m[10] == 1.0
            && self.m[11] == 0.0
            && self.m[12] == 0.0
            && self.m[13] == 0.0
            && self.m[14] == 0.0
            && self.m[15] == 1.0
    }

    pub fn is_zero(&self) -> bool {
        self.m.iter().all(|&v| v == 0.0)
    }

    pub fn add(&mut self, other: &Mat4) {
        for i in 0..MATRIX4_SIZE {
            self.m[i] += other.m[i];
        }
    }

    pub fn subtract(&mut self, other: &Mat4) {
        for i in 0..MATRIX4_SIZE {
            self.m[i] -= other.m[i];
        }
    }

    pub fn multiply_scalar(&mut self, scalar: f32) {
        for i in 0..MATRIX4_SIZE {
            self.m[i] *= scalar;
        }
    }

    pub fn transform(&mut self, vec: &Vec3) {
        let x = vec.x;
        let y = vec.y;
        let z = vec.z;

        self.m[12] = self.m[0] * x + self.m[4] * y + self.m[8] * z + self.m[12];
        self.m[13] = self.m[1] * x + self.m[5] * y + self.m[9] * z + self.m[13];
        self.m[14] = self.m[2] * x + self.m[6] * y + self.m[10] * z + self.m[14];
        self.m[15] = self.m[3] * x + self.m[7] * y + self.m[11] * z + self.m[15];
    }

    pub fn scale_vec(&mut self, vec: &Vec3) {
        let x = vec.x;
        let y = vec.y;
        let z = vec.z;

        self.m[0] *= x;
        self.m[1] *= x;
        self.m[2] *= x;
        self.m[3] *= x;
        self.m[4] *= y;
        self.m[5] *= y;
        self.m[6] *= y;
        self.m[7] *= y;
        self.m[8] *= z;
        self.m[9] *= z;
        self.m[10] *= z;
        self.m[11] *= z;
    }

    pub fn from_srt(q: &Quaternion, v: &Vec3, s: &Vec3) -> Mat4 {
        let x = q.x;
        let y = q.y;
        let z = q.z;
        let w = q.w;

        let x2 = x + x;
        let y2 = y + y;
        let z2 = z + z;

        let xx = x * x2;
        let xy = x * y2;
        let xz = x * z2;
        let yy = y * y2;
        let yz = y * z2;
        let zz = z * z2;
        let wx = w * x2;
        let wy = w * y2;
        let wz = w * z2;

        let sx = s.x;
        let sy = s.y;
        let sz = s.z;

        Mat4::new(
            (1.0 - (yy + zz)) * sx, (xy + wz) * sx, (xz - wy) * sx, 0.0,
            (xy - wz) * sy, (1.0 - (xx + zz)) * sy, (yz + wx) * sy, 0.0,
            (xz + wy) * sz, (yz - wx) * sz, (1.0 - (xx + yy)) * sz, 0.0,
            v.x, v.y, v.z, 1.0,
        )
    }

    pub fn get_translation(&self) -> Vec3 {
        Vec3::new(self.m[12], self.m[13], self.m[14])
    }

    pub fn get_scale(&self) -> Vec3 {
        let m00 = self.m[0];
        let m01 = self.m[1];
        let m02 = self.m[2];
        let m04 = self.m[4];
        let m05 = self.m[5];
        let m06 = self.m[6];
        let m08 = self.m[8];
        let m09 = self.m[9];
        let m10 = self.m[10];

        Vec3::new(
            (m00 * m00 + m01 * m01 + m02 * m02).sqrt(),
            (m04 * m04 + m05 * m05 + m06 * m06).sqrt(),
            (m08 * m08 + m09 * m09 + m10 * m10).sqrt(),
        )
    }

    pub fn get_rotation(&self) -> Quaternion {
        let trace = self.m[0] + self.m[5] + self.m[10];

        if trace > 0.0 {
            let s = (trace + 1.0).sqrt() * 2.0;
            Quaternion::new(
                (self.m[6] - self.m[9]) / s,
                (self.m[8] - self.m[2]) / s,
                (self.m[1] - self.m[4]) / s,
                0.25 * s,
            )
        } else if self.m[0] > self.m[5] && self.m[0] > self.m[10] {
            let s = (1.0 + self.m[0] - self.m[5] - self.m[10]).sqrt() * 2.0;
            Quaternion::new(
                0.25 * s,
                (self.m[1] + self.m[4]) / s,
                (self.m[8] + self.m[2]) / s,
                (self.m[6] - self.m[9]) / s,
            )
        } else if self.m[5] > self.m[10] {
            let s = (1.0 + self.m[5] - self.m[0] - self.m[10]).sqrt() * 2.0;
            Quaternion::new(
                (self.m[1] + self.m[4]) / s,
                0.25 * s,
                (self.m[6] + self.m[9]) / s,
                (self.m[8] - self.m[2]) / s,
            )
        } else {
            let s = (1.0 + self.m[10] - self.m[0] - self.m[5]).sqrt() * 2.0;
            Quaternion::new(
                (self.m[8] + self.m[2]) / s,
                (self.m[6] + self.m[9]) / s,
                0.25 * s,
                (self.m[1] - self.m[4]) / s,
            )
        }
    }

    pub fn equals(&self, other: &Mat4, epsilon: f32) -> bool {
        for i in 0..MATRIX4_SIZE {
            if (self.m[i] - other.m[i]).abs() > epsilon {
                return false;
            }
        }
        true
    }
}

impl Default for Mat4 {
    fn default() -> Self {
        Mat4::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    const FLOAT_CMP_PRECISION: f32 = 0.00001;

    fn assert_float_eq(a: f32, b: f32, epsilon: f32) {
        assert!(
            (a - b).abs() < epsilon,
            "Float values not equal: {} != {}",
            a,
            b
        );
    }

    fn assert_mat4_eq(a: &Mat4, b: &Mat4, epsilon: f32) {
        for i in 0..16 {
            assert_float_eq(a.m[i], b.m[i], epsilon);
        }
    }

    fn assert_vec3_eq(a: &Vec3, b: &Vec3, epsilon: f32) {
        assert_float_eq(a.x, b.x, epsilon);
        assert_float_eq(a.y, b.y, epsilon);
        assert_float_eq(a.z, b.z, epsilon);
    }

    fn assert_quat_eq(a: &Quaternion, b: &Quaternion, epsilon: f32) {
        assert_float_eq(a.x, b.x, epsilon);
        assert_float_eq(a.y, b.y, epsilon);
        assert_float_eq(a.z, b.z, epsilon);
        assert_float_eq(a.w, b.w, epsilon);
    }

    #[test]
    fn test_identity() {
        let m = Mat4::IDENTITY;
        assert!(m.is_identity());
        assert_float_eq(m.determinant(), 1.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_from_translation() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let m = Mat4::from_translation(&v);

        assert_float_eq(m.m[12], 1.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[13], 2.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[14], 3.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[0], 1.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[5], 1.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[10], 1.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[15], 1.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_from_scale() {
        let v = Vec3::new(2.0, 3.0, 4.0);
        let m = Mat4::from_scale(&v);

        assert_float_eq(m.m[0], 2.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[5], 3.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[10], 4.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[15], 1.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_from_quat_identity() {
        let q = Quaternion::IDENTITY;
        let m = Mat4::from_quat(&q);

        assert_mat4_eq(&m, &Mat4::IDENTITY, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_from_quat_rotation_x() {
        let angle = PI / 2.0;
        let q = Quaternion::from_axis_angle(&Vec3::UNIT_X, angle);
        let m = Mat4::from_quat(&q);

        assert_float_eq(m.m[0], 1.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[5], 0.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[6], 1.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[9], -1.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[10], 0.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_from_quat_rotation_y() {
        let angle = PI / 2.0;
        let q = Quaternion::from_axis_angle(&Vec3::UNIT_Y, angle);
        let m = Mat4::from_quat(&q);

        assert_float_eq(m.m[0], 0.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[2], -1.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[5], 1.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[8], 1.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[10], 0.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_from_quat_rotation_z() {
        let angle = PI / 2.0;
        let q = Quaternion::from_axis_angle(&Vec3::UNIT_Z, angle);
        let m = Mat4::from_quat(&q);

        assert_float_eq(m.m[0], 0.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[1], 1.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[4], -1.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[5], 0.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[10], 1.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_determinant_identity() {
        let m = Mat4::IDENTITY;
        assert_float_eq(m.determinant(), 1.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_determinant_scale() {
        let m = Mat4::from_scale(&Vec3::new(2.0, 3.0, 4.0));
        assert_float_eq(m.determinant(), 24.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_determinant_translation() {
        let m = Mat4::from_translation(&Vec3::new(1.0, 2.0, 3.0));
        assert_float_eq(m.determinant(), 1.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_determinant_rotation() {
        let q = Quaternion::from_axis_angle(&Vec3::new(1.0, 2.0, 3.0).get_normalized(), 0.5);
        let m = Mat4::from_quat(&q);
        assert_float_eq(m.determinant(), 1.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_invert_identity() {
        let mut m = Mat4::IDENTITY;
        m.invert();
        assert_mat4_eq(&m, &Mat4::IDENTITY, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_invert_translation() {
        let mut m = Mat4::from_translation(&Vec3::new(1.0, 2.0, 3.0));
        m.invert();

        assert_float_eq(m.m[12], -1.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[13], -2.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[14], -3.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_invert_scale() {
        let mut m = Mat4::from_scale(&Vec3::new(2.0, 4.0, 8.0));
        m.invert();

        assert_float_eq(m.m[0], 0.5, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[5], 0.25, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[10], 0.125, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_invert_multiply() {
        let original = Mat4::from_srt(
            &Quaternion::from_axis_angle(&Vec3::new(1.0, 2.0, 3.0).get_normalized(), 0.7),
            &Vec3::new(1.0, 2.0, 3.0),
            &Vec3::new(2.0, 3.0, 4.0),
        );

        let mut inv = original;
        inv.invert();

        let mut result = Mat4::ZERO;
        Mat4::multiply(&original, &inv, &mut result);

        assert_mat4_eq(&result, &Mat4::IDENTITY, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_invert_zero_det() {
        let mut m = Mat4::ZERO;
        m.invert();
        assert!(m.is_zero());
    }

    #[test]
    fn test_look_at_identity() {
        let eye = Vec3::new(0.0, 0.0, 0.0);
        let center = Vec3::new(0.0, 0.0, -1.0);
        let up = Vec3::UNIT_Y;

        let m = Mat4::look_at(&eye, &center, &up);

        assert_float_eq(m.m[12], 0.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[13], 0.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[14], 0.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_look_at_translation() {
        let eye = Vec3::new(0.0, 0.0, 5.0);
        let center = Vec3::new(0.0, 0.0, 0.0);
        let up = Vec3::UNIT_Y;

        let m = Mat4::look_at(&eye, &center, &up);

        assert_float_eq(m.m[14], -5.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_perspective() {
        let fovy = PI / 4.0;
        let aspect = 16.0 / 9.0;
        let near = 0.1;
        let far = 100.0;

        let m = Mat4::perspective(fovy, aspect, near, far);

        let f = 1.0 / (fovy / 2.0).tan();
        assert_float_eq(m.m[0], f / aspect, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[5], f, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[11], -1.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_orthographic() {
        let left = -1.0;
        let right = 1.0;
        let bottom = -1.0;
        let top = 1.0;
        let near = 0.0;
        let far = 100.0;

        let m = Mat4::orthographic(left, right, bottom, top, near, far);

        assert_float_eq(m.m[0], 1.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[5], 1.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[15], 1.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_multiply_identity() {
        let m = Mat4::from_translation(&Vec3::new(1.0, 2.0, 3.0));
        let mut result = Mat4::ZERO;
        Mat4::multiply(&m, &Mat4::IDENTITY, &mut result);
        assert_mat4_eq(&result, &m, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_multiply_translations() {
        let m1 = Mat4::from_translation(&Vec3::new(1.0, 0.0, 0.0));
        let m2 = Mat4::from_translation(&Vec3::new(0.0, 2.0, 0.0));

        let mut result = Mat4::ZERO;
        Mat4::multiply(&m1, &m2, &mut result);

        assert_float_eq(result.m[12], 1.0, FLOAT_CMP_PRECISION);
        assert_float_eq(result.m[13], 2.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_from_srt() {
        let q = Quaternion::IDENTITY;
        let v = Vec3::new(1.0, 2.0, 3.0);
        let s = Vec3::new(2.0, 3.0, 4.0);

        let m = Mat4::from_srt(&q, &v, &s);

        assert_float_eq(m.m[0], 2.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[5], 3.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[10], 4.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[12], 1.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[13], 2.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[14], 3.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_get_translation() {
        let m = Mat4::from_translation(&Vec3::new(1.0, 2.0, 3.0));
        let t = m.get_translation();
        assert_vec3_eq(&t, &Vec3::new(1.0, 2.0, 3.0), FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_get_scale() {
        let m = Mat4::from_scale(&Vec3::new(2.0, 3.0, 4.0));
        let s = m.get_scale();
        assert_vec3_eq(&s, &Vec3::new(2.0, 3.0, 4.0), FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_get_rotation_identity() {
        let m = Mat4::IDENTITY;
        let r = m.get_rotation();
        assert_quat_eq(&r, &Quaternion::IDENTITY, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_transpose() {
        let mut m = Mat4::new(
            1.0, 2.0, 3.0, 4.0,
            5.0, 6.0, 7.0, 8.0,
            9.0, 10.0, 11.0, 12.0,
            13.0, 14.0, 15.0, 16.0,
        );
        m.transpose();

        assert_float_eq(m.m[0], 1.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[1], 5.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[2], 9.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[3], 13.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[4], 2.0, FLOAT_CMP_PRECISION);
        assert_float_eq(m.m[5], 6.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_equals() {
        let m1 = Mat4::IDENTITY;
        let m2 = Mat4::IDENTITY;
        assert!(m1.equals(&m2, FLOAT_CMP_PRECISION));

        let m3 = Mat4::from_translation(&Vec3::new(0.000001, 0.0, 0.0));
        assert!(m1.equals(&m3, 0.00001));
        assert!(!m1.equals(&m3, 0.0000001));
    }

    #[test]
    fn test_from_rotation_alias() {
        let q = Quaternion::from_axis_angle(&Vec3::UNIT_X, PI / 4.0);
        let m1 = Mat4::from_rotation(&q);
        let m2 = Mat4::from_quat(&q);
        assert_mat4_eq(&m1, &m2, FLOAT_CMP_PRECISION);
    }
}
