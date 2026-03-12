/****************************************************************************
Rust port of Cocos Creator GFX Queue
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::QueueType;

#[derive(Debug, Clone)]
pub struct QueueInfo {
    pub queue_type: QueueType,
}

impl Default for QueueInfo {
    fn default() -> Self {
        QueueInfo {
            queue_type: QueueType::Graphics,
        }
    }
}

#[derive(Debug)]
pub struct GfxQueue {
    pub id: u32,
    pub info: QueueInfo,
    pub num_draw_calls: u32,
    pub num_instances: u32,
    pub num_tris: u32,
}

impl GfxQueue {
    pub fn new(id: u32, info: QueueInfo) -> Self {
        GfxQueue {
            id,
            info,
            num_draw_calls: 0,
            num_instances: 0,
            num_tris: 0,
        }
    }

    pub fn get_type(&self) -> QueueType {
        self.info.queue_type
    }
}
