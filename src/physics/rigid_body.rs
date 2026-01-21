/****************************************************************************
Rust port of Cocos Creator Physics Rigid Body
Original C++ version Copyright (c) 2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::RefCounted;
use crate::core::scene_graph::Node;
use crate::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidBodyType {
    Dynamic = 0,
    Kinematic = 1,
    Static = 2,
}

#[derive(Debug, Clone)]
pub struct RigidBody {
    pub node: Node,
}

impl RigidBody {
    pub fn new(node: Node) -> Self {
        RigidBody { node }
    }
}
