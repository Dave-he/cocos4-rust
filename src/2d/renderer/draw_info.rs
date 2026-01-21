/****************************************************************************
Rust port of Cocos Creator Render Draw Info
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::RefCounted;

pub struct RenderDrawInfo {
    pub stage: u32,
    pub use_model: bool,
}

impl RenderDrawInfo {
    pub fn new(stage: u32, use_model: bool) -> Self {
        RenderDrawInfo { stage, use_model }
    }
}
