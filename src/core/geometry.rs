/****************************************************************************
Rust port of Cocos Creator Geometry System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::math::Mat4;
use crate::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeEnum {
    ShapeAABB = 0,
    ShapeSphere = 1,
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
        let min_x = self.center.x
            - self
                .half_extents
                .x
                .min(other.center.x - other.half_extents.x);
        let min_y = self.center.y
            - self
                .half_extents
                .y
                .min(other.center.y - other.half_extents.y);
        let min_z = self.center.z
            - self
                .half_extents
                .z
                .min(other.center.z - other.half_extents.z);

        let max_x = self.center.x
            + self
                .half_extents
                .x
                .max(other.center.x + other.half_extents.x);
        let max_y = self.center.y
            + self
                .half_extents
                .y
                .max(other.center.y + other.half_extents.y);
        let max_z = self.center.z
            + self
                .half_extents
                .z
                .max(other.center.z + other.half_extents.z);

        self.center = Vec3::new(
            (min_x + max_x) * 0.5,
            (min_y + max_y) * 0.5,
            (min_z + max_z) * 0.5,
        );
        self.half_extents = Vec3::new(
            (max_x - min_x) * 0.5,
            (max_y - min_y) * 0.5,
            (max_z - min_z) * 0.5,
        );
    }

    pub fn merge_point(&mut self, point: &Vec3) {
        self.center.x =
            (self.center.x.min(point.x) + point.x.max(self.center.x + self.half_extents.x)) * 0.5;
        self.center.y =
            (self.center.y.min(point.y) + point.y.max(self.center.y + self.half_extents.y)) * 0.5;
        self.center.z =
            (self.center.z.min(point.z) + point.z.max(self.center.z + self.half_extents.z)) * 0.5;

        self.half_extents.x = self.half_extents.x.max((point.x - self.center.x).abs());
        self.half_extents.y = self.half_extents.y.max((point.y - self.center.y).abs());
        self.half_extents.z = self.half_extents.z.max((point.z - self.center.z).abs());
    }

    pub fn contains(&self, point: &Vec3) -> bool {
        point.x >= self.center.x - self.half_extents.x
            && point.x <= self.center.x + self.half_extents.x
            && point.y >= self.center.y - self.half_extents.y
            && point.y <= self.center.y + self.half_extents.y
            && point.z >= self.center.z - self.half_extents.z
            && point.z <= self.center.z + self.half_extents.z
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

    pub fn normalize(&mut self) {
        let len = self.normal.length();
        if len > 0.0 {
            self.normal.x /= len;
            self.normal.y /= len;
            self.normal.z /= len;
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
