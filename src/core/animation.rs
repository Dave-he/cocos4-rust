/****************************************************************************
Rust port of Cocos Creator Skeletal Animation Utilities
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::core::scene_graph::Node;
use crate::math::Mat4;

pub trait Texture {}

#[derive(Clone)]
pub struct RealTimeJointTexture {
    pub textures: Vec<std::rc::Weak<dyn Texture>>,
    pub buffer: Vec<u8>,
}

impl RealTimeJointTexture {
    pub fn new() -> Self {
        RealTimeJointTexture {
            textures: Vec::new(),
            buffer: Vec::new(),
        }
    }
}

impl Drop for RealTimeJointTexture {
    fn drop(&mut self) {
        self.textures.clear();
        self.buffer.clear();
    }
}

#[derive(Clone)]
pub struct IJointTransform {
    pub node: Option<*mut Node>,
    pub local: Mat4,
    pub world: Mat4,
    pub stamp: i32,
    pub parent: Option<std::rc::Weak<IJointTransform>>,
}

impl IJointTransform {
    pub fn new() -> Self {
        IJointTransform {
            node: None,
            local: Mat4::IDENTITY,
            world: Mat4::IDENTITY,
            stamp: -1,
            parent: None,
        }
    }
}

pub fn get_world_matrix(_transform: &mut IJointTransform, _stamp: i32) -> Mat4 {
    Mat4::IDENTITY
}

pub fn get_transform(_node: &Node, _root: &Node) -> Option<std::rc::Weak<IJointTransform>> {
    None
}

pub fn delete_transform(_node: &Node) {
    None
}
