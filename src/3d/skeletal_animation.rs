/****************************************************************************
Rust port of Cocos Creator 3D Skeletal Animation System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::RefCounted;
use crate::core::scene_graph::Node;

#[derive(Debug, Clone)]
pub struct SkeletalAnimationState {
    pub current_time: f32,
}

pub trait SkeletalAnimation: RefCounted {
    fn get_state(&self) -> SkeletalAnimationState;
}
