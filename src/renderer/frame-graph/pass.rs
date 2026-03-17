/****************************************************************************
Rust port of Cocos Creator Frame Graph Pass System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::RefCounted;

#[derive(Debug, Clone)]
pub struct AssInsertPoint {
    pub x: f32,
    pub y: f32,
}

pub trait PassNode: RefCounted {}
