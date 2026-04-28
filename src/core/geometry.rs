/****************************************************************************
Rust port of Cocos Creator Geometry System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::math::{Mat3, Mat4, Quaternion, Vec3, FLOAT_CMP_PRECISION};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeEnum {
    ShapeAABB = 0,
    ShapeSphere = 1,
    ShapeOBB = 2,
    ShapeRay = 3,
    ShapeLine = 4,
    ShapeFrustum = 5,
    ShapePlane = 6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaneIntersectResult {
    Intersect = 1,
    OutsideFront = 0,
    InsideBack = -1,
}

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub center: Vec3,
    pub half_extents: Vec3,
    #[allow(dead_code)]
    is_valid: bool,
}

impl AABB {
    pub fn new(px: f32, py: f32, pz: f32, hw: f32, hh: f32, hl: f32) -> Self {
        AABB {
            center: Vec3::new(px, py, pz),
            half_extents: Vec3::new(hw, hh, hl),
            is_valid: true,
        }
    }

    pub fn set(&mut self, px: f32, py: f32, pz: f32, hw: f32, hh: f32, hl: f32) {
        self.center.set(px, py, pz);
        self.half_extents.set(hw, hh, hl);
    }

    pub fn set_center(&mut self, x: f32, y: f32, z: f32) {
        self.center.set(x, y, z);
    }

    pub fn merge(&mut self, other: &AABB) {
        let (min1, max1) = self.get_boundary();
        let (min2, max2) = other.get_boundary();

        let new_min = Vec3::new(
            min1.x.min(min2.x),
            min1.y.min(min2.y),
            min1.z.min(min2.z),
        );
        let new_max = Vec3::new(
            max1.x.max(max2.x),
            max1.y.max(max2.y),
            max1.z.max(max2.z),
        );

        self.center = Vec3::new(
            (new_min.x + new_max.x) * 0.5,
            (new_min.y + new_max.y) * 0.5,
            (new_min.z + new_max.z) * 0.5,
        );
        self.half_extents = Vec3::new(
            (new_max.x - new_min.x) * 0.5,
            (new_max.y - new_min.y) * 0.5,
            (new_max.z - new_min.z) * 0.5,
        );
    }

    pub fn merge_point(&mut self, point: &Vec3) {
        let (min, max) = self.get_boundary();
        let new_min = Vec3::new(min.x.min(point.x), min.y.min(point.y), min.z.min(point.z));
        let new_max = Vec3::new(max.x.max(point.x), max.y.max(point.y), max.z.max(point.z));

        self.center = Vec3::new(
            (new_min.x + new_max.x) * 0.5,
            (new_min.y + new_max.y) * 0.5,
            (new_min.z + new_max.z) * 0.5,
        );
        self.half_extents = Vec3::new(
            (new_max.x - new_min.x) * 0.5,
            (new_max.y - new_min.y) * 0.5,
            (new_max.z - new_min.z) * 0.5,
        );
    }

    /// Create an AABB from min and max corner points.
    pub fn from_min_max(min: Vec3, max: Vec3) -> Self {
        let center = Vec3::new(
            (min.x + max.x) * 0.5,
            (min.y + max.y) * 0.5,
            (min.z + max.z) * 0.5,
        );
        let half_extents = Vec3::new(
            (max.x - min.x) * 0.5,
            (max.y - min.y) * 0.5,
            (max.z - min.z) * 0.5,
        );
        AABB { center, half_extents, is_valid: true }
    }

    /// Create an AABB from a center point and half-extents vector.
    pub fn from_center_extents(center: Vec3, half_extents: Vec3) -> Self {
        AABB { center, half_extents, is_valid: true }
    }

    pub fn contains(&self, point: &Vec3) -> bool {
        point.x >= self.center.x - self.half_extents.x
            && point.x <= self.center.x + self.half_extents.x
            && point.y >= self.center.y - self.half_extents.y
            && point.y <= self.center.y + self.half_extents.y
            && point.z >= self.center.z - self.half_extents.z
            && point.z <= self.center.z + self.half_extents.z
    }

    /// Alias for contains, using abs-distance comparison (handles floating-point edge cases).
    pub fn contains_point(&self, point: &Vec3) -> bool {
        (point.x - self.center.x).abs() <= self.half_extents.x
            && (point.y - self.center.y).abs() <= self.half_extents.y
            && (point.z - self.center.z).abs() <= self.half_extents.z
    }

    /// Test if two AABBs overlap.
    pub fn intersects(&self, other: &AABB) -> bool {
        let dx = (self.center.x - other.center.x).abs();
        let dy = (self.center.y - other.center.y).abs();
        let dz = (self.center.z - other.center.z).abs();
        dx <= self.half_extents.x + other.half_extents.x
            && dy <= self.half_extents.y + other.half_extents.y
            && dz <= self.half_extents.z + other.half_extents.z
    }

    pub fn get_boundary(&self) -> (Vec3, Vec3) {
        let min = self.center - self.half_extents;
        let max = self.center + self.half_extents;
        (min, max)
    }

    pub fn to_bounding_sphere(&self) -> Sphere {
        Sphere {
            center: self.center,
            radius: self.half_extents.length(),
        }
    }

    pub fn transform(&self, m: &Mat4) -> AABB {
        let new_center = self.center.transform_mat4(m);
        let new_half_extents = transform_extent_m4(&self.half_extents, m);
        AABB {
            center: new_center,
            half_extents: Vec3::new(
                new_half_extents.x.abs(),
                new_half_extents.y.abs(),
                new_half_extents.z.abs(),
            ),
            is_valid: true,
        }
    }
}

impl Default for AABB {
    fn default() -> Self {
        AABB {
            center: Vec3::ZERO,
            half_extents: Vec3::ZERO,
            is_valid: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Sphere { center, radius }
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Sphere {
            center: Vec3::ZERO,
            radius: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl Plane {
    pub fn new(normal: Vec3, distance: f32) -> Self {
        let mut n = normal;
        n.normalize();
        Plane {
            normal: n,
            distance,
        }
    }

    pub fn from_points(a: &Vec3, b: &Vec3, c: &Vec3) -> Self {
        let mut n = Vec3::cross_vecs(&(*b - *a), &(*c - *a));
        n.normalize();
        let d = n.dot(a);
        Plane {
            normal: n,
            distance: d,
        }
    }

    pub fn normalize(&mut self) {
        let len = self.normal.length();
        if len > 0.0 {
            self.normal.x /= len;
            self.normal.y /= len;
            self.normal.z /= len;
            self.distance /= len;
        }
    }
}

impl Default for Plane {
    fn default() -> Self {
        Plane {
            normal: Vec3::new(0.0, 1.0, 0.0),
            distance: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OBB {
    pub center: Vec3,
    pub half_extents: Vec3,
    pub orientation: Mat3,
}

impl OBB {
    pub fn new(cx: f32, cy: f32, cz: f32, hw: f32, hh: f32, hl: f32) -> Self {
        OBB {
            center: Vec3::new(cx, cy, cz),
            half_extents: Vec3::new(hw, hh, hl),
            orientation: Mat3::IDENTITY,
        }
    }

    pub fn new_with_orientation(
        cx: f32,
        cy: f32,
        cz: f32,
        hw: f32,
        hh: f32,
        hl: f32,
        orientation: Mat3,
    ) -> Self {
        OBB {
            center: Vec3::new(cx, cy, cz),
            half_extents: Vec3::new(hw, hh, hl),
            orientation,
        }
    }

    pub fn from_aabb(aabb: &AABB, rotation: &Quaternion) -> Self {
        OBB {
            center: aabb.center,
            half_extents: aabb.half_extents,
            orientation: Mat3::from_quat(rotation),
        }
    }

    pub fn set(&mut self, cx: f32, cy: f32, cz: f32, hw: f32, hh: f32, hl: f32) {
        self.center.set(cx, cy, cz);
        self.half_extents.set(hw, hh, hl);
    }

    pub fn set_orientation(&mut self, orientation: Mat3) {
        self.orientation = orientation;
    }

    pub fn set_orientation_from_quat(&mut self, q: &Quaternion) {
        self.orientation = Mat3::from_quat(q);
    }

    pub fn contains(&self, point: &Vec3) -> bool {
        let tmp = *point - self.center;
        let local = tmp.transform_mat3(&self.orientation.get_transposed());
        
        local.x.abs() <= self.half_extents.x
            && local.y.abs() <= self.half_extents.y
            && local.z.abs() <= self.half_extents.z
    }

    pub fn get_boundary(&self) -> (Vec3, Vec3) {
        let tmp = transform_extent_m3(&self.half_extents, &self.orientation);
        let min = self.center - tmp;
        let max = self.center + tmp;
        (min, max)
    }

    pub fn transform(&self, m: &Mat4, rot: &Quaternion, scale: &Vec3, out: &mut OBB) {
        out.center = self.center.transform_mat4(m);
        out.orientation = Mat3::from_quat(rot);
        out.half_extents = Vec3::multiply_vecs(&self.half_extents, scale);
    }
}

impl Default for OBB {
    fn default() -> Self {
        OBB {
            center: Vec3::ZERO,
            half_extents: Vec3::ONE,
            orientation: Mat3::IDENTITY,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(ox: f32, oy: f32, oz: f32, dx: f32, dy: f32, dz: f32) -> Self {
        let mut dir = Vec3::new(dx, dy, dz);
        dir.normalize();
        Ray {
            origin: Vec3::new(ox, oy, oz),
            direction: dir,
        }
    }

    pub fn from_points(origin: &Vec3, target: &Vec3) -> Self {
        let mut dir = *target - *origin;
        dir.normalize();
        Ray {
            origin: *origin,
            direction: dir,
        }
    }

    pub fn set(&mut self, ox: f32, oy: f32, oz: f32, dx: f32, dy: f32, dz: f32) {
        self.origin.set(ox, oy, oz);
        self.direction.set(dx, dy, dz);
        self.direction.normalize();
    }

    pub fn compute_hit(&self, distance: f32) -> Vec3 {
        self.origin + self.direction * distance
    }

    pub fn intersect_aabb(&self, aabb: &AABB) -> Option<f32> {
        let (min, max) = aabb.get_boundary();
        self.intersect_aabb_min_max(&min, &max)
    }

    fn intersect_aabb_min_max(&self, min: &Vec3, max: &Vec3) -> Option<f32> {
        let mut tmin: f32 = 0.0;
        let mut tmax: f32 = f32::MAX;

        for i in 0..3 {
            let (o, d, bmin, bmax) = match i {
                0 => (self.origin.x, self.direction.x, min.x, max.x),
                1 => (self.origin.y, self.direction.y, min.y, max.y),
                _ => (self.origin.z, self.direction.z, min.z, max.z),
            };

            if d.abs() < FLOAT_CMP_PRECISION {
                if o < bmin || o > bmax {
                    return None;
                }
            } else {
                let t1 = (bmin - o) / d;
                let t2 = (bmax - o) / d;
                let (t1, t2) = if t1 > t2 { (t2, t1) } else { (t1, t2) };
                tmin = tmin.max(t1);
                tmax = tmax.min(t2);
                if tmin > tmax {
                    return None;
                }
            }
        }

        if tmin > 0.0 {
            Some(tmin)
        } else {
            Some(tmax)
        }
    }

    pub fn intersect_sphere(&self, sphere: &Sphere) -> Option<f32> {
        let e = sphere.center - self.origin;
        let e_sq = e.length_squared();
        let a_length = e.dot(&self.direction);
        let f_sq = sphere.radius * sphere.radius - (e_sq - a_length * a_length);

        if f_sq < 0.0 {
            return None;
        }

        let f = f_sq.sqrt();
        let t = if e_sq < sphere.radius * sphere.radius {
            a_length + f
        } else {
            a_length - f
        };

        if t < 0.0 {
            None
        } else {
            Some(t)
        }
    }

    pub fn intersect_plane(&self, plane: &Plane) -> Option<f32> {
        let denom = plane.normal.dot(&self.direction);
        if denom.abs() < FLOAT_CMP_PRECISION {
            return None;
        }

        let d = point_plane_distance(&self.origin, plane);
        let t = -d / denom;

        if t < 0.0 {
            None
        } else {
            Some(t)
        }
    }

    pub fn intersect_obb(&self, obb: &OBB) -> Option<f32> {
        let size = [
            obb.half_extents.x,
            obb.half_extents.y,
            obb.half_extents.z,
        ];

        let x = Vec3::new(obb.orientation.m[0], obb.orientation.m[1], obb.orientation.m[2]);
        let y = Vec3::new(obb.orientation.m[3], obb.orientation.m[4], obb.orientation.m[5]);
        let z = Vec3::new(obb.orientation.m[6], obb.orientation.m[7], obb.orientation.m[8]);

        let p = obb.center - self.origin;

        let f = [
            x.dot(&self.direction),
            y.dot(&self.direction),
            z.dot(&self.direction),
        ];

        let e = [
            x.dot(&p),
            y.dot(&p),
            z.dot(&p),
        ];

        let mut t = [0.0f32; 6];

        for i in 0..3 {
            if f[i].abs() < FLOAT_CMP_PRECISION {
                if -e[i] - size[i] > 0.0 || -e[i] + size[i] < 0.0 {
                    return None;
                }
                t[i * 2] = f32::MIN;
                t[i * 2 + 1] = f32::MAX;
            } else {
                t[i * 2] = (e[i] + size[i]) / f[i];
                t[i * 2 + 1] = (e[i] - size[i]) / f[i];
            }
        }

        let tmin = t[0].min(t[1]).max(t[2].min(t[3])).max(t[4].min(t[5]));
        let tmax = t[0].max(t[1]).min(t[2].max(t[3])).min(t[4].max(t[5]));

        if tmax < 0.0 || tmin > tmax {
            return None;
        }

        Some(if tmin > 0.0 { tmin } else { tmax })
    }
}

impl Default for Ray {
    fn default() -> Self {
        Ray {
            origin: Vec3::ZERO,
            direction: Vec3::new(0.0, 0.0, -1.0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub start: Vec3,
    pub end: Vec3,
}

impl Line {
    pub fn new(sx: f32, sy: f32, sz: f32, ex: f32, ey: f32, ez: f32) -> Self {
        Line {
            start: Vec3::new(sx, sy, sz),
            end: Vec3::new(ex, ey, ez),
        }
    }

    pub fn from_points(start: &Vec3, end: &Vec3) -> Self {
        Line {
            start: *start,
            end: *end,
        }
    }

    pub fn set(&mut self, sx: f32, sy: f32, sz: f32, ex: f32, ey: f32, ez: f32) {
        self.start.set(sx, sy, sz);
        self.end.set(ex, ey, ez);
    }

    pub fn length(&self) -> f32 {
        self.start.distance(&self.end)
    }

    pub fn direction(&self) -> Vec3 {
        let mut dir = self.end - self.start;
        dir.normalize();
        dir
    }

    pub fn center(&self) -> Vec3 {
        (self.start + self.end) * 0.5
    }
}

impl Default for Line {
    fn default() -> Self {
        Line {
            start: Vec3::ZERO,
            end: Vec3::new(0.0, 0.0, -1.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Frustum {
    pub planes: [Plane; 6],
    pub vertices: [Vec3; 8],
}

impl Frustum {
    pub fn new() -> Self {
        Frustum {
            planes: [
                Plane::default(),
                Plane::default(),
                Plane::default(),
                Plane::default(),
                Plane::default(),
                Plane::default(),
            ],
            vertices: [
                Vec3::ZERO,
                Vec3::ZERO,
                Vec3::ZERO,
                Vec3::ZERO,
                Vec3::ZERO,
                Vec3::ZERO,
                Vec3::ZERO,
                Vec3::ZERO,
            ],
        }
    }

    pub fn update(&mut self, m: &Mat4, inv: &Mat4) {
        let planes = &mut self.planes;
        
        planes[0].normal = Vec3::new(m.m[3] + m.m[0], m.m[7] + m.m[4], m.m[11] + m.m[8]);
        planes[0].distance = -(m.m[15] + m.m[12]);
        
        planes[1].normal = Vec3::new(m.m[3] - m.m[0], m.m[7] - m.m[4], m.m[11] - m.m[8]);
        planes[1].distance = -(m.m[15] - m.m[12]);
        
        planes[2].normal = Vec3::new(m.m[3] + m.m[1], m.m[7] + m.m[5], m.m[11] + m.m[9]);
        planes[2].distance = -(m.m[15] + m.m[13]);
        
        planes[3].normal = Vec3::new(m.m[3] - m.m[1], m.m[7] - m.m[5], m.m[11] - m.m[9]);
        planes[3].distance = -(m.m[15] - m.m[13]);
        
        planes[4].normal = Vec3::new(m.m[3] + m.m[2], m.m[7] + m.m[6], m.m[11] + m.m[10]);
        planes[4].distance = -(m.m[15] + m.m[14]);
        
        planes[5].normal = Vec3::new(m.m[3] - m.m[2], m.m[7] - m.m[6], m.m[11] - m.m[10]);
        planes[5].distance = -(m.m[15] - m.m[14]);

        for pl in planes.iter_mut().take(6) {
            let inv_dist = 1.0 / pl.normal.length();
            pl.normal.x *= inv_dist;
            pl.normal.y *= inv_dist;
            pl.normal.z *= inv_dist;
            pl.distance *= inv_dist;
        }

        let corners = [
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(-1.0, 1.0, 1.0),
            Vec3::new(-1.0, -1.0, 1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(1.0, 1.0, -1.0),
            Vec3::new(-1.0, 1.0, -1.0),
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, -1.0),
        ];

        for (i, corner) in corners.iter().enumerate() {
            self.vertices[i] = corner.transform_mat4(inv);
        }
    }

    pub fn update_planes(&mut self) {
        let v = &self.vertices;
        let p = &mut self.planes;

        p[0] = Plane::from_points(&v[1], &v[6], &v[5]);
        p[1] = Plane::from_points(&v[3], &v[4], &v[7]);
        p[2] = Plane::from_points(&v[6], &v[3], &v[7]);
        p[3] = Plane::from_points(&v[0], &v[5], &v[4]);
        p[4] = Plane::from_points(&v[2], &v[0], &v[3]);
        p[5] = Plane::from_points(&v[7], &v[5], &v[6]);
    }

    pub fn create_perspective(
        &mut self,
        aspect: f32,
        fov: f32,
        near: f32,
        far: f32,
        transform: &Mat4,
    ) {
        let h = (fov * 0.5).tan();
        let w = h * aspect;

        let near_temp = Vec3::new(near * w, near * h, near);
        let far_temp = Vec3::new(far * w, far * h, far);

        let v = &mut self.vertices;

        v[0] = Vec3::new(near_temp.x, near_temp.y, -near_temp.z).transform_mat4(transform);
        v[1] = Vec3::new(-near_temp.x, near_temp.y, -near_temp.z).transform_mat4(transform);
        v[2] = Vec3::new(-near_temp.x, -near_temp.y, -near_temp.z).transform_mat4(transform);
        v[3] = Vec3::new(near_temp.x, -near_temp.y, -near_temp.z).transform_mat4(transform);

        v[4] = Vec3::new(far_temp.x, far_temp.y, -far_temp.z).transform_mat4(transform);
        v[5] = Vec3::new(-far_temp.x, far_temp.y, -far_temp.z).transform_mat4(transform);
        v[6] = Vec3::new(-far_temp.x, -far_temp.y, -far_temp.z).transform_mat4(transform);
        v[7] = Vec3::new(far_temp.x, -far_temp.y, -far_temp.z).transform_mat4(transform);

        self.update_planes();
    }

    pub fn create_orthographic(
        &mut self,
        width: f32,
        height: f32,
        near: f32,
        far: f32,
        transform: &Mat4,
    ) {
        let half_width = width / 2.0;
        let half_height = height / 2.0;

        let v = &mut self.vertices;

        v[0] = Vec3::new(half_width, half_height, -near).transform_mat4(transform);
        v[1] = Vec3::new(-half_width, half_height, -near).transform_mat4(transform);
        v[2] = Vec3::new(-half_width, -half_height, -near).transform_mat4(transform);
        v[3] = Vec3::new(half_width, -half_height, -near).transform_mat4(transform);
        v[4] = Vec3::new(half_width, half_height, -far).transform_mat4(transform);
        v[5] = Vec3::new(-half_width, half_height, -far).transform_mat4(transform);
        v[6] = Vec3::new(-half_width, -half_height, -far).transform_mat4(transform);
        v[7] = Vec3::new(half_width, -half_height, -far).transform_mat4(transform);

        self.update_planes();
    }

    pub fn contains_aabb(&self, aabb: &AABB) -> bool {
        for plane in &self.planes {
            if aabb_plane(aabb, plane) == PlaneIntersectResult::InsideBack {
                return false;
            }
        }
        true
    }

    pub fn contains_sphere(&self, sphere: &Sphere) -> bool {
        for plane in &self.planes {
            if sphere_plane(sphere, plane) == PlaneIntersectResult::InsideBack {
                return false;
            }
        }
        true
    }
}

impl Default for Frustum {
    fn default() -> Self {
        Frustum::new()
    }
}

fn transform_extent_m3(extent: &Vec3, m3: &Mat3) -> Vec3 {
    Vec3::new(
        m3.m[0].abs() * extent.x + m3.m[3].abs() * extent.y + m3.m[6].abs() * extent.z,
        m3.m[1].abs() * extent.x + m3.m[4].abs() * extent.y + m3.m[7].abs() * extent.z,
        m3.m[2].abs() * extent.x + m3.m[5].abs() * extent.y + m3.m[8].abs() * extent.z,
    )
}

fn transform_extent_m4(extent: &Vec3, m4: &Mat4) -> Vec3 {
    Vec3::new(
        m4.m[0].abs() * extent.x + m4.m[4].abs() * extent.y + m4.m[8].abs() * extent.z,
        m4.m[1].abs() * extent.x + m4.m[5].abs() * extent.y + m4.m[9].abs() * extent.z,
        m4.m[2].abs() * extent.x + m4.m[6].abs() * extent.y + m4.m[10].abs() * extent.z,
    )
}

pub fn point_plane_distance(point: &Vec3, plane: &Plane) -> f32 {
    plane.normal.dot(point) - plane.distance
}

pub fn aabb_plane(aabb: &AABB, plane: &Plane) -> PlaneIntersectResult {
    let r = aabb.half_extents.x * plane.normal.x.abs()
        + aabb.half_extents.y * plane.normal.y.abs()
        + aabb.half_extents.z * plane.normal.z.abs();
    let dot = plane.normal.dot(&aabb.center);

    if dot + r < plane.distance {
        PlaneIntersectResult::InsideBack
    } else if dot - r > plane.distance {
        PlaneIntersectResult::OutsideFront
    } else {
        PlaneIntersectResult::Intersect
    }
}

pub fn sphere_plane(sphere: &Sphere, plane: &Plane) -> PlaneIntersectResult {
    let dot = plane.normal.dot(&sphere.center);
    let r = sphere.radius * plane.normal.length();

    if dot + r < plane.distance {
        PlaneIntersectResult::InsideBack
    } else if dot - r > plane.distance {
        PlaneIntersectResult::OutsideFront
    } else {
        PlaneIntersectResult::Intersect
    }
}

pub fn aabb_aabb(a: &AABB, b: &AABB) -> bool {
    let (a_min, a_max) = a.get_boundary();
    let (b_min, b_max) = b.get_boundary();

    a_min.x <= b_max.x && a_max.x >= b_min.x
        && a_min.y <= b_max.y && a_max.y >= b_min.y
        && a_min.z <= b_max.z && a_max.z >= b_min.z
}

pub fn sphere_sphere(a: &Sphere, b: &Sphere) -> bool {
    let r = a.radius + b.radius;
    a.center.distance_squared(&b.center) < r * r
}

pub fn sphere_aabb(sphere: &Sphere, aabb: &AABB) -> bool {
    let closest = closest_point_on_aabb(&sphere.center, aabb);
    sphere.center.distance_squared(&closest) < sphere.radius * sphere.radius
}

fn closest_point_on_aabb(point: &Vec3, aabb: &AABB) -> Vec3 {
    let (min, max) = aabb.get_boundary();
    Vec3::new(
        point.x.clamp(min.x, max.x),
        point.y.clamp(min.y, max.y),
        point.z.clamp(min.z, max.z),
    )
}

pub fn obb_obb(a: &OBB, b: &OBB) -> bool {
    let test: [Vec3; 15] = [
        Vec3::new(a.orientation.m[0], a.orientation.m[1], a.orientation.m[2]),
        Vec3::new(a.orientation.m[3], a.orientation.m[4], a.orientation.m[5]),
        Vec3::new(a.orientation.m[6], a.orientation.m[7], a.orientation.m[8]),
        Vec3::new(b.orientation.m[0], b.orientation.m[1], b.orientation.m[2]),
        Vec3::new(b.orientation.m[3], b.orientation.m[4], b.orientation.m[5]),
        Vec3::new(b.orientation.m[6], b.orientation.m[7], b.orientation.m[8]),
        Vec3::cross_vecs(
            &Vec3::new(a.orientation.m[0], a.orientation.m[1], a.orientation.m[2]),
            &Vec3::new(b.orientation.m[0], b.orientation.m[1], b.orientation.m[2]),
        ),
        Vec3::cross_vecs(
            &Vec3::new(a.orientation.m[0], a.orientation.m[1], a.orientation.m[2]),
            &Vec3::new(b.orientation.m[3], b.orientation.m[4], b.orientation.m[5]),
        ),
        Vec3::cross_vecs(
            &Vec3::new(a.orientation.m[0], a.orientation.m[1], a.orientation.m[2]),
            &Vec3::new(b.orientation.m[6], b.orientation.m[7], b.orientation.m[8]),
        ),
        Vec3::cross_vecs(
            &Vec3::new(a.orientation.m[3], a.orientation.m[4], a.orientation.m[5]),
            &Vec3::new(b.orientation.m[0], b.orientation.m[1], b.orientation.m[2]),
        ),
        Vec3::cross_vecs(
            &Vec3::new(a.orientation.m[3], a.orientation.m[4], a.orientation.m[5]),
            &Vec3::new(b.orientation.m[3], b.orientation.m[4], b.orientation.m[5]),
        ),
        Vec3::cross_vecs(
            &Vec3::new(a.orientation.m[3], a.orientation.m[4], a.orientation.m[5]),
            &Vec3::new(b.orientation.m[6], b.orientation.m[7], b.orientation.m[8]),
        ),
        Vec3::cross_vecs(
            &Vec3::new(a.orientation.m[6], a.orientation.m[7], a.orientation.m[8]),
            &Vec3::new(b.orientation.m[0], b.orientation.m[1], b.orientation.m[2]),
        ),
        Vec3::cross_vecs(
            &Vec3::new(a.orientation.m[6], a.orientation.m[7], a.orientation.m[8]),
            &Vec3::new(b.orientation.m[3], b.orientation.m[4], b.orientation.m[5]),
        ),
        Vec3::cross_vecs(
            &Vec3::new(a.orientation.m[6], a.orientation.m[7], a.orientation.m[8]),
            &Vec3::new(b.orientation.m[6], b.orientation.m[7], b.orientation.m[8]),
        ),
    ];

    let a_vertices = get_obb_vertices(a);
    let b_vertices = get_obb_vertices(b);

    for axis in &test {
        if axis.length_squared() < FLOAT_CMP_PRECISION {
            continue;
        }

        let a_interval = get_interval(&a_vertices, axis);
        let b_interval = get_interval(&b_vertices, axis);

        if b_interval.0 > a_interval.1 || a_interval.0 > b_interval.1 {
            return false;
        }
    }

    true
}

fn get_obb_vertices(obb: &OBB) -> [Vec3; 8] {
    let c = obb.center;
    let e = obb.half_extents;
    
    let a1 = Vec3::new(obb.orientation.m[0], obb.orientation.m[1], obb.orientation.m[2]);
    let a2 = Vec3::new(obb.orientation.m[3], obb.orientation.m[4], obb.orientation.m[5]);
    let a3 = Vec3::new(obb.orientation.m[6], obb.orientation.m[7], obb.orientation.m[8]);

    [
        c + a1 * e.x + a2 * e.y + a3 * e.z,
        c - a1 * e.x + a2 * e.y + a3 * e.z,
        c + a1 * e.x - a2 * e.y + a3 * e.z,
        c + a1 * e.x + a2 * e.y - a3 * e.z,
        c - a1 * e.x - a2 * e.y - a3 * e.z,
        c + a1 * e.x - a2 * e.y - a3 * e.z,
        c - a1 * e.x + a2 * e.y - a3 * e.z,
        c - a1 * e.x - a2 * e.y + a3 * e.z,
    ]
}

fn get_interval(vertices: &[Vec3; 8], axis: &Vec3) -> (f32, f32) {
    let mut min = axis.dot(&vertices[0]);
    let mut max = min;

    for vertex in vertices.iter().take(8).skip(1) {
        let projection = axis.dot(vertex);
        min = min.min(projection);
        max = max.max(projection);
    }

    (min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_float_eq(a: f32, b: f32, epsilon: f32) {
        assert!(
            (a - b).abs() < epsilon,
            "Float values not equal: {} != {}",
            a,
            b
        );
    }

    fn assert_vec3_eq(a: &Vec3, b: &Vec3, epsilon: f32) {
        assert_float_eq(a.x, b.x, epsilon);
        assert_float_eq(a.y, b.y, epsilon);
        assert_float_eq(a.z, b.z, epsilon);
    }

    #[test]
    fn test_aabb_new() {
        let aabb = AABB::new(1.0, 2.0, 3.0, 0.5, 0.5, 0.5);
        assert_eq!(aabb.center, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(aabb.half_extents, Vec3::new(0.5, 0.5, 0.5));
    }

    #[test]
    fn test_aabb_contains() {
        let aabb = AABB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        assert!(aabb.contains(&Vec3::new(0.0, 0.0, 0.0)));
        assert!(aabb.contains(&Vec3::new(0.5, 0.5, 0.5)));
        assert!(aabb.contains(&Vec3::new(-1.0, -1.0, -1.0)));
        assert!(aabb.contains(&Vec3::new(1.0, 1.0, 1.0)));
        assert!(!aabb.contains(&Vec3::new(1.1, 0.0, 0.0)));
    }

    #[test]
    fn test_aabb_get_boundary() {
        let aabb = AABB::new(0.0, 0.0, 0.0, 1.0, 2.0, 3.0);
        let (min, max) = aabb.get_boundary();
        assert_vec3_eq(&min, &Vec3::new(-1.0, -2.0, -3.0), FLOAT_CMP_PRECISION);
        assert_vec3_eq(&max, &Vec3::new(1.0, 2.0, 3.0), FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_aabb_merge() {
        let mut aabb1 = AABB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let aabb2 = AABB::new(2.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        aabb1.merge(&aabb2);
        
        assert_vec3_eq(&aabb1.center, &Vec3::new(1.0, 0.0, 0.0), FLOAT_CMP_PRECISION);
        assert_vec3_eq(&aabb1.half_extents, &Vec3::new(2.0, 1.0, 1.0), FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_aabb_aabb_intersection() {
        let aabb1 = AABB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let aabb2 = AABB::new(1.5, 0.0, 0.0, 1.0, 1.0, 1.0);
        let aabb3 = AABB::new(3.0, 0.0, 0.0, 1.0, 1.0, 1.0);

        assert!(aabb_aabb(&aabb1, &aabb2));
        assert!(!aabb_aabb(&aabb1, &aabb3));
    }

    #[test]
    fn test_sphere_new() {
        let sphere = Sphere::new(Vec3::new(1.0, 2.0, 3.0), 5.0);
        assert_eq!(sphere.center, Vec3::new(1.0, 2.0, 3.0));
        assert_float_eq(sphere.radius, 5.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_sphere_sphere_intersection() {
        let sphere1 = Sphere::new(Vec3::new(0.0, 0.0, 0.0), 1.0);
        let sphere2 = Sphere::new(Vec3::new(1.5, 0.0, 0.0), 1.0);
        let sphere3 = Sphere::new(Vec3::new(3.0, 0.0, 0.0), 1.0);

        assert!(sphere_sphere(&sphere1, &sphere2));
        assert!(!sphere_sphere(&sphere1, &sphere3));
    }

    #[test]
    fn test_sphere_aabb_intersection() {
        let sphere = Sphere::new(Vec3::new(0.0, 0.0, 0.0), 1.0);
        let aabb1 = AABB::new(0.5, 0.0, 0.0, 0.5, 0.5, 0.5);
        let aabb2 = AABB::new(2.0, 0.0, 0.0, 0.5, 0.5, 0.5);

        assert!(sphere_aabb(&sphere, &aabb1));
        assert!(!sphere_aabb(&sphere, &aabb2));
    }

    #[test]
    fn test_plane_new() {
        let plane = Plane::new(Vec3::new(0.0, 1.0, 0.0), 5.0);
        assert_vec3_eq(&plane.normal, &Vec3::new(0.0, 1.0, 0.0), FLOAT_CMP_PRECISION);
        assert_float_eq(plane.distance, 5.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_plane_from_points() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(1.0, 0.0, 0.0);
        let c = Vec3::new(0.0, 1.0, 0.0);
        let plane = Plane::from_points(&a, &b, &c);
        
        assert_vec3_eq(&plane.normal, &Vec3::new(0.0, 0.0, 1.0), FLOAT_CMP_PRECISION);
        assert_float_eq(plane.distance, 0.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_obb_new() {
        let obb = OBB::new(1.0, 2.0, 3.0, 0.5, 0.5, 0.5);
        assert_vec3_eq(&obb.center, &Vec3::new(1.0, 2.0, 3.0), FLOAT_CMP_PRECISION);
        assert_vec3_eq(&obb.half_extents, &Vec3::new(0.5, 0.5, 0.5), FLOAT_CMP_PRECISION);
        assert!(obb.orientation.is_identity());
    }

    #[test]
    fn test_obb_contains() {
        let obb = OBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        assert!(obb.contains(&Vec3::new(0.0, 0.0, 0.0)));
        assert!(obb.contains(&Vec3::new(0.5, 0.5, 0.5)));
        assert!(!obb.contains(&Vec3::new(1.5, 0.0, 0.0)));
    }

    #[test]
    fn test_obb_obb_intersection() {
        let obb1 = OBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let obb2 = OBB::new(1.5, 0.0, 0.0, 1.0, 1.0, 1.0);
        let obb3 = OBB::new(3.0, 0.0, 0.0, 1.0, 1.0, 1.0);

        assert!(obb_obb(&obb1, &obb2));
        assert!(!obb_obb(&obb1, &obb3));
    }

    #[test]
    fn test_ray_new() {
        let ray = Ray::new(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        assert_vec3_eq(&ray.origin, &Vec3::ZERO, FLOAT_CMP_PRECISION);
        
        let expected_dir = Vec3::new(0.0, 0.0, 1.0);
        assert_vec3_eq(&ray.direction, &expected_dir, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_ray_from_points() {
        let origin = Vec3::new(0.0, 0.0, 0.0);
        let target = Vec3::new(0.0, 0.0, 10.0);
        let ray = Ray::from_points(&origin, &target);
        
        assert_vec3_eq(&ray.origin, &origin, FLOAT_CMP_PRECISION);
        assert_vec3_eq(&ray.direction, &Vec3::new(0.0, 0.0, 1.0), FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_ray_compute_hit() {
        let ray = Ray::new(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        let hit = ray.compute_hit(5.0);
        assert_vec3_eq(&hit, &Vec3::new(0.0, 0.0, 5.0), FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_ray_intersect_aabb() {
        let ray = Ray::new(0.0, 0.0, -5.0, 0.0, 0.0, 1.0);
        let aabb = AABB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        
        let result = ray.intersect_aabb(&aabb);
        assert!(result.is_some());
        assert_float_eq(result.unwrap(), 4.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_ray_intersect_aabb_miss() {
        let ray = Ray::new(0.0, 0.0, -5.0, 0.0, 0.0, 1.0);
        let aabb = AABB::new(5.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        
        let result = ray.intersect_aabb(&aabb);
        assert!(result.is_none());
    }

    #[test]
    fn test_ray_intersect_sphere() {
        let ray = Ray::new(0.0, 0.0, -5.0, 0.0, 0.0, 1.0);
        let sphere = Sphere::new(Vec3::new(0.0, 0.0, 0.0), 1.0);
        
        let result = ray.intersect_sphere(&sphere);
        assert!(result.is_some());
        assert_float_eq(result.unwrap(), 4.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_ray_intersect_sphere_miss() {
        let ray = Ray::new(0.0, 0.0, -5.0, 0.0, 0.0, 1.0);
        let sphere = Sphere::new(Vec3::new(5.0, 0.0, 0.0), 1.0);
        
        let result = ray.intersect_sphere(&sphere);
        assert!(result.is_none());
    }

    #[test]
    fn test_ray_intersect_plane() {
        let ray = Ray::new(0.0, 0.0, -5.0, 0.0, 0.0, 1.0);
        let plane = Plane::new(Vec3::new(0.0, 0.0, 1.0), 0.0);
        
        let result = ray.intersect_plane(&plane);
        assert!(result.is_some());
        assert_float_eq(result.unwrap(), 5.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_ray_intersect_plane_parallel() {
        let ray = Ray::new(0.0, 0.0, -5.0, 0.0, 0.0, 1.0);
        let plane = Plane::new(Vec3::new(1.0, 0.0, 0.0), 0.0);
        
        let result = ray.intersect_plane(&plane);
        assert!(result.is_none());
    }

    #[test]
    fn test_ray_intersect_obb() {
        let ray = Ray::new(0.0, 0.0, -5.0, 0.0, 0.0, 1.0);
        let obb = OBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        
        let result = ray.intersect_obb(&obb);
        assert!(result.is_some());
        assert_float_eq(result.unwrap(), 4.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_ray_intersect_obb_miss() {
        let ray = Ray::new(0.0, 0.0, -5.0, 0.0, 0.0, 1.0);
        let obb = OBB::new(5.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        
        let result = ray.intersect_obb(&obb);
        assert!(result.is_none());
    }

    #[test]
    fn test_line_new() {
        let line = Line::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        assert_vec3_eq(&line.start, &Vec3::ZERO, FLOAT_CMP_PRECISION);
        assert_vec3_eq(&line.end, &Vec3::new(1.0, 1.0, 1.0), FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_line_length() {
        let line = Line::new(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        assert_float_eq(line.length(), 1.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_line_direction() {
        let line = Line::new(0.0, 0.0, 0.0, 0.0, 0.0, 5.0);
        let dir = line.direction();
        assert_vec3_eq(&dir, &Vec3::new(0.0, 0.0, 1.0), FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_line_center() {
        let line = Line::new(0.0, 0.0, 0.0, 2.0, 2.0, 2.0);
        let center = line.center();
        assert_vec3_eq(&center, &Vec3::new(1.0, 1.0, 1.0), FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_frustum_new() {
        let frustum = Frustum::new();
        for plane in &frustum.planes {
            assert_vec3_eq(&plane.normal, &Vec3::new(0.0, 1.0, 0.0), FLOAT_CMP_PRECISION);
        }
        for vertex in &frustum.vertices {
            assert_vec3_eq(vertex, &Vec3::ZERO, FLOAT_CMP_PRECISION);
        }
    }

    #[test]
    fn test_frustum_contains_aabb() {
        let mut frustum = Frustum::new();
        
        frustum.planes[0] = Plane::new(Vec3::new(-1.0, 0.0, 0.0), -1.0);
        frustum.planes[1] = Plane::new(Vec3::new(1.0, 0.0, 0.0), -1.0);
        frustum.planes[2] = Plane::new(Vec3::new(0.0, -1.0, 0.0), -1.0);
        frustum.planes[3] = Plane::new(Vec3::new(0.0, 1.0, 0.0), -1.0);
        frustum.planes[4] = Plane::new(Vec3::new(0.0, 0.0, -1.0), -1.0);
        frustum.planes[5] = Plane::new(Vec3::new(0.0, 0.0, 1.0), -1.0);

        let aabb_inside = AABB::new(0.0, 0.0, 0.0, 0.5, 0.5, 0.5);
        assert!(frustum.contains_aabb(&aabb_inside));

        let aabb_outside = AABB::new(2.0, 0.0, 0.0, 0.5, 0.5, 0.5);
        assert!(!frustum.contains_aabb(&aabb_outside));
    }

    #[test]
    fn test_frustum_contains_sphere() {
        let mut frustum = Frustum::new();
        
        frustum.planes[0] = Plane::new(Vec3::new(-1.0, 0.0, 0.0), -1.0);
        frustum.planes[1] = Plane::new(Vec3::new(1.0, 0.0, 0.0), -1.0);
        frustum.planes[2] = Plane::new(Vec3::new(0.0, -1.0, 0.0), -1.0);
        frustum.planes[3] = Plane::new(Vec3::new(0.0, 1.0, 0.0), -1.0);
        frustum.planes[4] = Plane::new(Vec3::new(0.0, 0.0, -1.0), -1.0);
        frustum.planes[5] = Plane::new(Vec3::new(0.0, 0.0, 1.0), -1.0);

        let sphere_inside = Sphere::new(Vec3::new(0.0, 0.0, 0.0), 0.5);
        assert!(frustum.contains_sphere(&sphere_inside));

        let sphere_outside = Sphere::new(Vec3::new(2.0, 0.0, 0.0), 0.5);
        assert!(!frustum.contains_sphere(&sphere_outside));
    }

    #[test]
    fn test_aabb_plane() {
        let aabb = AABB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        
        let plane_intersect = Plane::new(Vec3::new(0.0, 1.0, 0.0), 0.0);
        assert_eq!(aabb_plane(&aabb, &plane_intersect), PlaneIntersectResult::Intersect);
        
        let plane_above = Plane::new(Vec3::new(0.0, 1.0, 0.0), 3.0);
        assert_eq!(aabb_plane(&aabb, &plane_above), PlaneIntersectResult::InsideBack);
        
        let plane_outside_front = Plane::new(Vec3::new(0.0, 1.0, 0.0), -3.0);
        assert_eq!(aabb_plane(&aabb, &plane_outside_front), PlaneIntersectResult::OutsideFront);
    }

    #[test]
    fn test_sphere_plane() {
        let sphere = Sphere::new(Vec3::new(0.0, 0.0, 0.0), 1.0);
        
        let plane_intersect = Plane::new(Vec3::new(0.0, 1.0, 0.0), 0.0);
        assert_eq!(sphere_plane(&sphere, &plane_intersect), PlaneIntersectResult::Intersect);
        
        let plane_above = Plane::new(Vec3::new(0.0, 1.0, 0.0), 2.0);
        assert_eq!(sphere_plane(&sphere, &plane_above), PlaneIntersectResult::InsideBack);
        
        let plane_outside_front = Plane::new(Vec3::new(0.0, 1.0, 0.0), -2.0);
        assert_eq!(sphere_plane(&sphere, &plane_outside_front), PlaneIntersectResult::OutsideFront);
    }

    #[test]
    fn test_aabb_to_bounding_sphere() {
        let aabb = AABB::new(0.0, 0.0, 0.0, 1.0, 2.0, 3.0);
        let sphere = aabb.to_bounding_sphere();
        
        assert_vec3_eq(&sphere.center, &aabb.center, FLOAT_CMP_PRECISION);
        let expected_radius = (1.0_f32.powi(2) + 2.0_f32.powi(2) + 3.0_f32.powi(2)).sqrt();
        assert_float_eq(sphere.radius, expected_radius, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_ray_intersect_aabb_from_inside() {
        let ray = Ray::new(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        let aabb = AABB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        
        let result = ray.intersect_aabb(&aabb);
        assert!(result.is_some());
        assert_float_eq(result.unwrap(), 1.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_ray_intersect_sphere_from_inside() {
        let ray = Ray::new(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        let sphere = Sphere::new(Vec3::new(0.0, 0.0, 0.0), 2.0);
        
        let result = ray.intersect_sphere(&sphere);
        assert!(result.is_some());
        assert_float_eq(result.unwrap(), 2.0, FLOAT_CMP_PRECISION);
    }

    #[test]
    fn test_obb_get_boundary() {
        let obb = OBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let (min, max) = obb.get_boundary();
        
        assert_vec3_eq(&min, &Vec3::new(-1.0, -1.0, -1.0), FLOAT_CMP_PRECISION);
        assert_vec3_eq(&max, &Vec3::new(1.0, 1.0, 1.0), FLOAT_CMP_PRECISION);
    }
}

// ---------------------------------------------------------------------------
// Capsule
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapsuleAxis {
    X = 0,
    Y = 1,
    Z = 2,
}

#[derive(Debug, Clone)]
pub struct Capsule {
    pub radius: f32,
    pub half_height: f32,
    pub axis: CapsuleAxis,
    pub center: Vec3,
    pub rotation: Quaternion,
    pub ellipse_center0: Vec3,
    pub ellipse_center1: Vec3,
}

impl Capsule {
    pub fn new(radius: f32, half_height: f32, axis: CapsuleAxis) -> Self {
        let (ec0, ec1) = match axis {
            CapsuleAxis::X => (Vec3::new(half_height, 0.0, 0.0), Vec3::new(-half_height, 0.0, 0.0)),
            CapsuleAxis::Y => (Vec3::new(0.0, half_height, 0.0), Vec3::new(0.0, -half_height, 0.0)),
            CapsuleAxis::Z => (Vec3::new(0.0, 0.0, half_height), Vec3::new(0.0, 0.0, -half_height)),
        };
        Capsule {
            radius,
            half_height,
            axis,
            center: Vec3::ZERO,
            rotation: Quaternion::IDENTITY,
            ellipse_center0: ec0,
            ellipse_center1: ec1,
        }
    }

    pub fn transform(&self, m: &Mat4, _pos: &Vec3, rot: &Quaternion, scale: &Vec3) -> Capsule {
        let max_scale = scale.x.abs().max(scale.y.abs()).max(scale.z.abs());
        let new_center = self.center.transform_mat4(m);
        let new_radius = self.radius * max_scale;
        let new_half_height = self.half_height * max_scale;
        let new_rot = Quaternion::multiply(&self.rotation, rot);
        let (ec0, ec1) = match self.axis {
            CapsuleAxis::X => (Vec3::new(new_half_height, 0.0, 0.0), Vec3::new(-new_half_height, 0.0, 0.0)),
            CapsuleAxis::Y => (Vec3::new(0.0, new_half_height, 0.0), Vec3::new(0.0, -new_half_height, 0.0)),
            CapsuleAxis::Z => (Vec3::new(0.0, 0.0, new_half_height), Vec3::new(0.0, 0.0, -new_half_height)),
        };
        Capsule {
            radius: new_radius,
            half_height: new_half_height,
            axis: self.axis,
            center: new_center,
            rotation: new_rot,
            ellipse_center0: ec0,
            ellipse_center1: ec1,
        }
    }
}

impl Default for Capsule {
    fn default() -> Self {
        Self::new(0.5, 0.5, CapsuleAxis::Y)
    }
}

// ---------------------------------------------------------------------------
// Triangle
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
}

impl Triangle {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ax: f32, ay: f32, az: f32,
        bx: f32, by: f32, bz: f32,
        cx: f32, cy: f32, cz: f32,
    ) -> Self {
        Triangle {
            a: Vec3::new(ax, ay, az),
            b: Vec3::new(bx, by, bz),
            c: Vec3::new(cx, cy, cz),
        }
    }

    pub fn from_points(a: Vec3, b: Vec3, c: Vec3) -> Self {
        Triangle { a, b, c }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set(
        &mut self,
        ax: f32, ay: f32, az: f32,
        bx: f32, by: f32, bz: f32,
        cx: f32, cy: f32, cz: f32,
    ) {
        self.a.set(ax, ay, az);
        self.b.set(bx, by, bz);
        self.c.set(cx, cy, cz);
    }

    pub fn normal(&self) -> Vec3 {
        let ab = self.b - self.a;
        let ac = self.c - self.a;
        let mut n = Vec3::cross_vecs(&ab, &ac);
        n.normalize();
        n
    }

    pub fn area(&self) -> f32 {
        let ab = self.b - self.a;
        let ac = self.c - self.a;
        Vec3::cross_vecs(&ab, &ac).length() * 0.5
    }
}

impl Default for Triangle {
    fn default() -> Self {
        Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(1.0, 0.0, 0.0),
            c: Vec3::new(0.0, 1.0, 0.0),
        }
    }
}

// ---------------------------------------------------------------------------
// Spline
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplineMode {
    Linear = 0,
    Bezier = 1,
    CatmullRom = 2,
}

pub const SPLINE_WHOLE_INDEX: u32 = 0xFFFFFFFF;

#[derive(Debug, Clone)]
pub struct Spline {
    mode: SplineMode,
    knots: Vec<Vec3>,
}

impl Spline {
    pub fn new(mode: SplineMode) -> Self {
        Spline { mode, knots: Vec::new() }
    }

    pub fn get_mode(&self) -> SplineMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: SplineMode) {
        self.mode = mode;
    }

    pub fn get_knots(&self) -> &[Vec3] {
        &self.knots
    }

    pub fn set_knots(&mut self, knots: Vec<Vec3>) {
        self.knots = knots;
    }

    pub fn add_knot(&mut self, knot: Vec3) {
        self.knots.push(knot);
    }

    pub fn insert_knot(&mut self, index: u32, knot: Vec3) {
        self.knots.insert(index as usize, knot);
    }

    pub fn remove_knot(&mut self, index: u32) {
        if (index as usize) < self.knots.len() {
            self.knots.remove(index as usize);
        }
    }

    pub fn set_knot(&mut self, index: u32, knot: Vec3) {
        if let Some(k) = self.knots.get_mut(index as usize) {
            *k = knot;
        }
    }

    pub fn get_knot(&self, index: u32) -> Option<&Vec3> {
        self.knots.get(index as usize)
    }

    pub fn get_knot_count(&self) -> u32 {
        self.knots.len() as u32
    }

    pub fn clear_knots(&mut self) {
        self.knots.clear();
    }

    pub fn get_point(&self, t: f32, index: u32) -> Vec3 {
        let t = t.clamp(0.0, 1.0);
        match self.mode {
            SplineMode::Linear => self.get_point_linear(t, index),
            SplineMode::Bezier => self.get_point_bezier(t, index),
            SplineMode::CatmullRom => self.get_point_catmull_rom(t, index),
        }
    }

    pub fn get_points(&self, num: u32, index: u32) -> Vec<Vec3> {
        if num < 2 {
            return Vec::new();
        }
        (0..num).map(|i| self.get_point(i as f32 / (num - 1) as f32, index)).collect()
    }

    #[allow(dead_code)]
    fn get_segments(&self) -> u32 {
        match self.mode {
            SplineMode::Linear => self.knots.len().saturating_sub(1) as u32,
            SplineMode::Bezier => (self.knots.len() / 4) as u32,
            SplineMode::CatmullRom => self.knots.len().saturating_sub(1) as u32,
        }
    }

    fn get_point_linear(&self, t: f32, seg_index: u32) -> Vec3 {
        let n = self.knots.len();
        if n < 2 { return Vec3::ZERO; }
        let segments = (n - 1) as f32;
        let (v0, v1) = if seg_index == SPLINE_WHOLE_INDEX {
            let s = t * segments;
            let i = (s as usize).min(n - 2);
            let _local_t = s - i as f32;
            (self.knots[i], self.knots[i + 1])
        } else {
            let i = (seg_index as usize).min(n - 2);
            (self.knots[i], self.knots[i + 1])
        };
        Self::lerp_vec3(&v0, &v1, if seg_index == SPLINE_WHOLE_INDEX {
            t * segments - (t * segments) as u32 as f32
        } else { t })
    }

    fn get_point_bezier(&self, t: f32, seg_index: u32) -> Vec3 {
        let n = self.knots.len();
        if n < 4 { return Vec3::ZERO; }
        let seg_count = n / 4;
        let i = if seg_index == SPLINE_WHOLE_INDEX {
            ((t * seg_count as f32) as usize).min(seg_count - 1) * 4
        } else {
            ((seg_index as usize) * 4).min(n - 4)
        };
        let local_t = if seg_index == SPLINE_WHOLE_INDEX {
            (t * seg_count as f32).fract()
        } else { t };
        Self::calc_bezier(&self.knots[i], &self.knots[i+1], &self.knots[i+2], &self.knots[i+3], local_t)
    }

    fn get_point_catmull_rom(&self, t: f32, seg_index: u32) -> Vec3 {
        let n = self.knots.len();
        if n < 2 { return Vec3::ZERO; }
        let segments = (n - 1) as f32;
        let s = if seg_index == SPLINE_WHOLE_INDEX { t * segments } else { t };
        let i = (s as usize).clamp(0, n - 2);
        let local_t = s - i as f32;
        let p0 = self.knots[i.saturating_sub(1)];
        let p1 = self.knots[i];
        let p2 = self.knots[(i + 1).min(n - 1)];
        let p3 = self.knots[(i + 2).min(n - 1)];
        Self::calc_catmull_rom(&p0, &p1, &p2, &p3, local_t)
    }

    fn lerp_vec3(a: &Vec3, b: &Vec3, t: f32) -> Vec3 {
        Vec3::new(
            a.x + (b.x - a.x) * t,
            a.y + (b.y - a.y) * t,
            a.z + (b.z - a.z) * t,
        )
    }

    fn calc_bezier(v0: &Vec3, v1: &Vec3, v2: &Vec3, v3: &Vec3, t: f32) -> Vec3 {
        let u = 1.0 - t;
        let b0 = u * u * u;
        let b1 = 3.0 * u * u * t;
        let b2 = 3.0 * u * t * t;
        let b3 = t * t * t;
        Vec3::new(
            b0 * v0.x + b1 * v1.x + b2 * v2.x + b3 * v3.x,
            b0 * v0.y + b1 * v1.y + b2 * v2.y + b3 * v3.y,
            b0 * v0.z + b1 * v1.z + b2 * v2.z + b3 * v3.z,
        )
    }

    fn calc_catmull_rom(v0: &Vec3, v1: &Vec3, v2: &Vec3, v3: &Vec3, t: f32) -> Vec3 {
        let t2 = t * t;
        let t3 = t2 * t;
        let f0 = -0.5 * t3 + t2 - 0.5 * t;
        let f1 = 1.5 * t3 - 2.5 * t2 + 1.0;
        let f2 = -1.5 * t3 + 2.0 * t2 + 0.5 * t;
        let f3 = 0.5 * t3 - 0.5 * t2;
        Vec3::new(
            f0 * v0.x + f1 * v1.x + f2 * v2.x + f3 * v3.x,
            f0 * v0.y + f1 * v1.y + f2 * v2.y + f3 * v3.y,
            f0 * v0.z + f1 * v1.z + f2 * v2.z + f3 * v3.z,
        )
    }
}

impl Default for Spline {
    fn default() -> Self {
        Self::new(SplineMode::CatmullRom)
    }
}

// ---------------------------------------------------------------------------
// AnimationCurve (Keyframe + Hermite interpolation)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default)]
pub struct Keyframe {
    pub time: f32,
    pub value: f32,
    pub in_tangent: f32,
    pub out_tangent: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrapMode {
    Default = 0,
    Normal = 1,
    Loop = 2,
    PingPong = 3,
    Clamp = 4,
}

#[derive(Debug, Clone)]
pub struct AnimationCurve {
    pub keyframes: Vec<Keyframe>,
    pub pre_wrap_mode: WrapMode,
    pub post_wrap_mode: WrapMode,
}

impl AnimationCurve {
    pub fn new() -> Self {
        AnimationCurve {
            keyframes: Vec::new(),
            pre_wrap_mode: WrapMode::Loop,
            post_wrap_mode: WrapMode::Clamp,
        }
    }

    pub fn constant(value: f32) -> Self {
        AnimationCurve {
            keyframes: vec![
                Keyframe { time: 0.0, value, in_tangent: 0.0, out_tangent: 0.0 },
                Keyframe { time: 1.0, value, in_tangent: 0.0, out_tangent: 0.0 },
            ],
            pre_wrap_mode: WrapMode::Clamp,
            post_wrap_mode: WrapMode::Clamp,
        }
    }

    pub fn add_key(&mut self, kf: Keyframe) {
        let pos = self.keyframes.partition_point(|k| k.time <= kf.time);
        self.keyframes.insert(pos, kf);
    }

    pub fn evaluate(&self, time: f32) -> f32 {
        let n = self.keyframes.len();
        if n == 0 { return 0.0; }
        if n == 1 { return self.keyframes[0].value; }

        let start = self.keyframes[0].time;
        let end = self.keyframes[n - 1].time;
        let duration = end - start;

        let t = if duration <= 0.0 {
            start
        } else {
            match self.post_wrap_mode {
                WrapMode::Loop => start + ((time - start) % duration + duration) % duration,
                WrapMode::PingPong => {
                    let cycle = ((time - start) / duration).abs();
                    if (cycle as u32).is_multiple_of(2) {
                        start + (time - start).abs() % duration
                    } else {
                        end - (time - start).abs() % duration
                    }
                }
                _ => time.clamp(start, end),
            }
        };

        let pos = self.keyframes.partition_point(|k| k.time <= t);
        let right = pos.min(n - 1);
        let left = right.saturating_sub(1);
        if left == right { return self.keyframes[left].value; }

        let lk = &self.keyframes[left];
        let rk = &self.keyframes[right];
        let dx = rk.time - lk.time;
        if dx < 1e-7 { return lk.value; }
        let s = (t - lk.time) / dx;
        Self::hermite(lk.value, lk.out_tangent * dx, rk.value, rk.in_tangent * dx, s)
    }

    fn hermite(p0: f32, m0: f32, p1: f32, m1: f32, t: f32) -> f32 {
        let t2 = t * t;
        let t3 = t2 * t;
        (2.0 * t3 - 3.0 * t2 + 1.0) * p0
            + (t3 - 2.0 * t2 + t) * m0
            + (-2.0 * t3 + 3.0 * t2) * p1
            + (t3 - t2) * m1
    }
}

impl Default for AnimationCurve {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod geometry_extra_tests {
    use super::*;

    #[test]
    fn test_capsule_default() {
        let c = Capsule::default();
        assert_eq!(c.radius, 0.5);
        assert_eq!(c.half_height, 0.5);
        assert_eq!(c.axis, CapsuleAxis::Y);
    }

    #[test]
    fn test_triangle_normal() {
        let t = Triangle::new(
            0.0, 0.0, 0.0,
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
        );
        let n = t.normal();
        assert!((n.z - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_triangle_area() {
        let t = Triangle::new(
            0.0, 0.0, 0.0,
            2.0, 0.0, 0.0,
            0.0, 2.0, 0.0,
        );
        assert!((t.area() - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_spline_linear() {
        let mut s = Spline::new(SplineMode::Linear);
        s.add_knot(Vec3::ZERO);
        s.add_knot(Vec3::new(1.0, 0.0, 0.0));
        let mid = s.get_point(0.5, SPLINE_WHOLE_INDEX);
        assert!((mid.x - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_spline_catmull_rom() {
        let mut s = Spline::new(SplineMode::CatmullRom);
        s.add_knot(Vec3::ZERO);
        s.add_knot(Vec3::new(1.0, 0.0, 0.0));
        s.add_knot(Vec3::new(2.0, 0.0, 0.0));
        let p = s.get_point(1.0, SPLINE_WHOLE_INDEX);
        assert!((p.x - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_animation_curve_constant() {
        let curve = AnimationCurve::constant(3.0);
        assert!((curve.evaluate(0.0) - 3.0).abs() < 1e-5);
        assert!((curve.evaluate(0.5) - 3.0).abs() < 1e-5);
        assert!((curve.evaluate(1.0) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_animation_curve_add_key_and_evaluate() {
        let mut curve = AnimationCurve::new();
        curve.add_key(Keyframe { time: 0.0, value: 0.0, in_tangent: 0.0, out_tangent: 1.0 });
        curve.add_key(Keyframe { time: 1.0, value: 1.0, in_tangent: 1.0, out_tangent: 0.0 });
        let v = curve.evaluate(0.5);
        assert!(v > 0.0 && v < 1.0);
    }

    #[test]
    fn test_spline_knot_ops() {
        let mut s = Spline::new(SplineMode::Linear);
        s.add_knot(Vec3::ZERO);
        s.add_knot(Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(s.get_knot_count(), 2);
        s.remove_knot(0);
        assert_eq!(s.get_knot_count(), 1);
    }
}
