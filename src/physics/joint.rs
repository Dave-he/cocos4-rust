use crate::base::RefCounted;
use crate::core::scene_graph::BaseNode;
use crate::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JointType {
    Sphere = 0,
    Revolute = 1,
    Spherical = 2,
    Prismatic = 3,
    Fixed = 4,
    Distance = 5,
    D6Spring = 6,
    D6Joint = 7,
}

pub trait Joint: RefCounted {}

pub trait RigidBody: RefCounted {}

pub trait Shape: RefCounted {}

pub trait World: RefCounted {}
