/****************************************************************************
Rust port of Cocos Creator 2D Sprite Component Stub
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::label::Label;
use super::srite::SpriteFrame;
use crate::base::RefCounted;

pub struct Sprite {
    pub node: Node,
    pub label: Label,
}

impl Sprite {
    pub fn new() -> Self {
        Sprite {
            node: Node::default(),
            label: Label::default(),
        }
    }
}

impl Default for Sprite {
    fn default() -> Self {
        Self::new()
    }
}
