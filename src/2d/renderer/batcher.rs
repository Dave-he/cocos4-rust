/****************************************************************************
Rust port of Cocos Creator 2D Batcher System
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::RefCounted;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Batcher2DType {
    Dynamic = 0,
    World = 1,
}

pub struct Batcher2D {
    pub batcher_2d_type: Batcher2DType,
}

impl Batcher2D {
    pub fn new() -> Self {
        Batcher2D {
            batcher_2d_type: Batcher2DType::Dynamic,
        }
    }
}

impl Default for Batcher2D {
    fn default() -> Self {
        Self::new()
    }
}
