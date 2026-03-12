use super::{Mat4, Quaternion, Vec3};

const MATRIX3_SIZE: usize = 9;
const EPSILON: f32 = 0.000001;

#[repr(C)]
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
            m: [
                m00, m01, m02, m03, m04, m05, m06, m07, m08,
            ],
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

    pub fn from_quat(q: &Quaternion) -> Self {
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

        Mat3 {
            m: [
                1.0 - yy - zz,
                yx + wz,
                zx - wy,
                yx - wz,
                1.0 - xx - zz,
                zy + wx,
                zx + wy,
                zy - wx,
                1.0 - xx - yy,
            ],
        }
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

    pub fn identity(&mut self) {
        self.m.copy_from_slice(&Mat3::IDENTITY.m);
    }

    pub fn transpose(&mut self) {
        if self.m[1] == self.m[3] && self.m[2] == self.m[6] && self.m[5] == self.m[7] {
            return;
        }

        let a01 = self.m[1];
        let a02 = self.m[2];
        let a12 = self.m[5];

        self.m[1] = self.m[3];
        self.m[2] = self.m[6];
        self.m[3] = a01;
        self.m[5] = self.m[7];
        self.m[6] = a02;
        self.m[7] = a12;
    }

    pub fn get_transposed(&self) -> Mat3 {
        let mut result = *self;
        result.transpose();
        result
    }

    pub fn invert(&mut self) {
        let a00 = self.m[0];
        let a01 = self.m[1];
        let a02 = self.m[2];
        let a10 = self.m[3];
        let a11 = self.m[4];
        let a12 = self.m[5];
        let a20 = self.m[6];
        let a21 = self.m[7];
        let a22 = self.m[8];

        let b01 = a22 * a11 - a12 * a21;
        let b11 = -a22 * a10 + a12 * a20;
        let b21 = a21 * a10 - a11 * a20;

        let det = a00 * b01 + a01 * b11 + a02 * b21;

        if det.abs() < EPSILON {
            self.m = [0.0; MATRIX3_SIZE];
            return;
        }

        let inv_det = 1.0 / det;

        self.m[0] = b01 * inv_det;
        self.m[1] = (-a22 * a01 + a02 * a21) * inv_det;
        self.m[2] = (a12 * a01 - a02 * a11) * inv_det;
        self.m[3] = b11 * inv_det;
        self.m[4] = (a22 * a00 - a02 * a20) * inv_det;
        self.m[5] = (-a12 * a00 + a02 * a10) * inv_det;
        self.m[6] = b21 * inv_det;
        self.m[7] = (-a21 * a00 + a01 * a20) * inv_det;
        self.m[8] = (a11 * a00 - a01 * a10) * inv_det;
    }

    pub fn get_inverted(&self) -> Mat3 {
        let mut result = *self;
        result.invert();
        result
    }

    pub fn determinant(&self) -> f32 {
        let a00 = self.m[0];
        let a01 = self.m[1];
        let a02 = self.m[2];
        let a10 = self.m[3];
        let a11 = self.m[4];
        let a12 = self.m[5];
        let a20 = self.m[6];
        let a21 = self.m[7];
        let a22 = self.m[8];

        a00 * (a22 * a11 - a12 * a21)
            + a01 * (-a22 * a10 + a12 * a20)
            + a02 * (a21 * a10 - a11 * a20)
    }

    pub fn multiply(&mut self, other: &Mat3) {
        let a00 = self.m[0];
        let a01 = self.m[1];
        let a02 = self.m[2];
        let a10 = self.m[3];
        let a11 = self.m[4];
        let a12 = self.m[5];
        let a20 = self.m[6];
        let a21 = self.m[7];
        let a22 = self.m[8];

        let b00 = other.m[0];
        let b01 = other.m[1];
        let b02 = other.m[2];
        let b10 = other.m[3];
        let b11 = other.m[4];
        let b12 = other.m[5];
        let b20 = other.m[6];
        let b21 = other.m[7];
        let b22 = other.m[8];

        self.m[0] = b00 * a00 + b01 * a10 + b02 * a20;
        self.m[1] = b00 * a01 + b01 * a11 + b02 * a21;
        self.m[2] = b00 * a02 + b01 * a12 + b02 * a22;
        self.m[3] = b10 * a00 + b11 * a10 + b12 * a20;
        self.m[4] = b10 * a01 + b11 * a11 + b12 * a21;
        self.m[5] = b10 * a02 + b11 * a12 + b12 * a22;
        self.m[6] = b20 * a00 + b21 * a10 + b22 * a20;
        self.m[7] = b20 * a01 + b21 * a11 + b22 * a21;
        self.m[8] = b20 * a02 + b21 * a12 + b22 * a22;
    }

    pub fn multiply_mat3(a: &Mat3, b: &Mat3) -> Mat3 {
        let mut result = *a;
        result.multiply(b);
        result
    }

    pub fn multiply_vec3(&self, vec: &Vec3) -> Vec3 {
        let x = vec.x;
        let y = vec.y;
        let z = vec.z;

        Vec3 {
            x: self.m[0] * x + self.m[3] * y + self.m[6] * z,
            y: self.m[1] * x + self.m[4] * y + self.m[7] * z,
            z: self.m[2] * x + self.m[5] * y + self.m[8] * z,
        }
    }

    pub fn scale(&mut self, x: f32, y: f32) {
        self.m[0] *= x;
        self.m[1] *= x;
        self.m[2] *= x;
        self.m[3] *= y;
        self.m[4] *= y;
        self.m[5] *= y;
    }

    pub fn rotate(&mut self, angle: f32) {
        let s = angle.sin();
        let c = angle.cos();

        let a00 = self.m[0];
        let a01 = self.m[1];
        let a02 = self.m[2];
        let a10 = self.m[3];
        let a11 = self.m[4];
        let a12 = self.m[5];

        self.m[0] = c * a00 + s * a10;
        self.m[1] = c * a01 + s * a11;
        self.m[2] = c * a02 + s * a12;
        self.m[3] = c * a10 - s * a00;
        self.m[4] = c * a11 - s * a01;
        self.m[5] = c * a12 - s * a02;
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.m[6] = self.m[0] * x + self.m[3] * y + self.m[6];
        self.m[7] = self.m[1] * x + self.m[4] * y + self.m[7];
        self.m[8] = self.m[2] * x + self.m[5] * y + self.m[8];
    }

    pub fn from_translation(x: f32, y: f32) -> Mat3 {
        let mut result = Mat3::IDENTITY;
        result.translate(x, y);
        result
    }

    pub fn from_rotation(angle: f32) -> Mat3 {
        let mut result = Mat3::IDENTITY;
        result.rotate(angle);
        result
    }

    pub fn from_scale(x: f32, y: f32) -> Mat3 {
        let mut result = Mat3::IDENTITY;
        result.scale(x, y);
        result
    }

    pub fn from_view_up(view: &Vec3, up: Option<&Vec3>) -> Mat3 {
        let default_up = Vec3::new(0.0, 1.0, 0.0);
        let up = up.unwrap_or(&default_up);

        if view.length_squared() < EPSILON * EPSILON {
            return Mat3::IDENTITY;
        }

        let mut right = Vec3::cross_vecs(up, view);
        right.normalize();

        if right.length_squared() < EPSILON * EPSILON {
            return Mat3::IDENTITY;
        }

        let adjusted_up = Vec3::cross_vecs(view, &right);

        Mat3::new(
            right.x, right.y, right.z,
            adjusted_up.x, adjusted_up.y, adjusted_up.z,
            view.x, view.y, view.z,
        )
    }

    pub fn is_identity(&self) -> bool {
        self.m[0] == 1.0
            && self.m[1] == 0.0
            && self.m[2] == 0.0
            && self.m[3] == 0.0
            && self.m[4] == 1.0
            && self.m[5] == 0.0
            && self.m[6] == 0.0
            && self.m[7] == 0.0
            && self.m[8] == 1.0
    }

    pub fn is_zero(&self) -> bool {
        self.m.iter().all(|&v| v == 0.0)
    }

    pub fn add(&mut self, other: &Mat3) {
        for i in 0..MATRIX3_SIZE {
            self.m[i] += other.m[i];
        }
    }

    pub fn subtract(&mut self, other: &Mat3) {
        for i in 0..MATRIX3_SIZE {
            self.m[i] -= other.m[i];
        }
    }

    pub fn multiply_scalar(&mut self, scalar: f32) {
        for i in 0..MATRIX3_SIZE {
            self.m[i] *= scalar;
        }
    }

    /// 从 Mat4 提取旋转矩阵
    pub fn from_mat4(m: &Mat4) -> Mat3 {
        Mat3::new(
            m.m[0], m.m[1], m.m[2],
            m.m[4], m.m[5], m.m[6],
            m.m[8], m.m[9], m.m[10],
        )
    }

    /// 根据视口的前方向和上方向计算矩阵
    pub fn from_view_up(view: &Vec3, up: &Vec3) -> Mat3 {
        let mut z = *view;
        z.normalize();

        let mut x = Vec3::cross_vecs(up, &z);
        x.normalize();

        let y = Vec3::cross_vecs(&z, &x);

        Mat3::new(
            x.x, x.y, x.z,
            y.x, y.y, y.z,
            z.x, z.y, z.z,
        )
    }

    /// 根据本地坐标轴朝向计算矩阵
    pub fn from_axes(x_axis: &Vec3, y_axis: &Vec3, z_axis: &Vec3) -> Mat3 {
        Mat3::new(
            x_axis.x, x_axis.y, x_axis.z,
            y_axis.x, y_axis.y, y_axis.z,
            z_axis.x, z_axis.y, z_axis.z,
        )
    }

    /// 将矩阵转换为欧拉角，旋转顺序为 YXZ
    pub fn to_euler(&self) -> Vec3 {
        let m00 = self.m[0];
        let m01 = self.m[1];
        let m02 = self.m[2];
        let m10 = self.m[3];
        let m11 = self.m[4];
        let m12 = self.m[5];
        let m20 = self.m[6];
        let m21 = self.m[7];
        let m22 = self.m[8];

        let x = m12.atan2(m22);
        let y = (-m02).asin();
        let z = m01.atan2(m00);

        Vec3::new(x, y, z)
    }
}

impl Default for Mat3 {
    fn default() -> Self {
        Mat3::ZERO
    }
}
