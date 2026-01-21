/****************************************************************************
Rust port of Cocos Creator Physics Shape
Original C++ version Copyright (c) 2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::RefCounted;
use crate::math::Vec3;

#[derive(Debug, Clone)]
pub struct Box {
    pub center: Vec3,
    pub half_extents: Vec3,
}

impl Box {
    pub fn new(center: Vec3, half_extents: Vec3) -> Self {
        Box {
            center,
            half_extents,
        }
    }
}

impl Default for Box {
    fn default() -> Self {
        Box {
            center: Vec3::ZERO,
            half_extents: Vec3::ZERO,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sphere {
    pub radius: f32,
}

impl Sphere {
    pub fn new(radius: f32) -> Self {
        Sphere { radius }
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Sphere { radius: 1.0 }
    }
}

pub trait Shape: RefCounted {
    fn get_type(&self) -> ShapeType;
    fn get_box(&self) -> &Box;
    fn get_sphere(&self) -> &Sphere;
}
