/****************************************************************************
Rust port of Cocos Creator Physics Rigid Body
Original C++ version Copyright (c) 2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::math::Vec3;
use std::sync::{Arc, Weak};
use crate::core::scene_graph::Node;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidBodyType {
    Dynamic = 0,
    Kinematic = 1,
    Static = 2,
}

#[derive(Debug, Clone)]
pub struct RigidBody {
    pub node: Option<Weak<dyn Node>>,
    pub body_type: RigidBodyType,
    pub mass: f32,
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
}

impl RigidBody {
    pub fn new() -> Self {
        RigidBody {
            node: None,
            body_type: RigidBodyType::Dynamic,
            mass: 1.0,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
        }
    }

    pub fn with_node(node: Weak<dyn Node>) -> Self {
        RigidBody {
            node: Some(node),
            body_type: RigidBodyType::Dynamic,
            mass: 1.0,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
        }
    }

    pub fn set_body_type(&mut self, body_type: RigidBodyType) {
        self.body_type = body_type;
    }

    pub fn set_mass(&mut self, mass: f32) {
        self.mass = mass;
    }

    pub fn set_velocity(&mut self, velocity: Vec3) {
        self.velocity = velocity;
    }

    pub fn set_angular_velocity(&mut self, angular_velocity: Vec3) {
        self.angular_velocity = angular_velocity;
    }
}

impl Default for RigidBody {
    fn default() -> Self {
        Self::new()
    }
}
