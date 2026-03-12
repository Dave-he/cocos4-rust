/****************************************************************************
Rust port of Cocos Creator Physics Character Controller
Original C++ version Copyright (c) 2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::RefCounted;
use crate::core::scene_graph::BaseNode;
use crate::math::Vec3;

#[derive(Debug, Clone)]
pub struct BoxCharacterController {
    pub radius: f32,
    pub height: f32,
}

impl BoxCharacterController {
    pub fn new(radius: f32, height: f32) -> Self {
        BoxCharacterController { radius, height }
    }
}

pub trait CharacterController: RefCounted {}
