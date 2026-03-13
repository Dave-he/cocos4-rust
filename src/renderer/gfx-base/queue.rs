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

    pub fn submit_commands(&mut self, draw_calls: u32, instances: u32, tris: u32) {
        self.num_draw_calls += draw_calls;
        self.num_instances += instances;
        self.num_tris += tris;
    }

    pub fn wait_idle(&self) {}

    pub fn reset_stats(&mut self) {
        self.num_draw_calls = 0;
        self.num_instances = 0;
        self.num_tris = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_new() {
        let q = GfxQueue::new(1, QueueInfo::default());
        assert_eq!(q.id, 1);
        assert_eq!(q.get_type(), QueueType::Graphics);
        assert_eq!(q.num_draw_calls, 0);
    }

    #[test]
    fn test_queue_submit() {
        let mut q = GfxQueue::new(1, QueueInfo::default());
        q.submit_commands(5, 10, 100);
        assert_eq!(q.num_draw_calls, 5);
        assert_eq!(q.num_instances, 10);
        assert_eq!(q.num_tris, 100);
    }

    #[test]
    fn test_queue_reset_stats() {
        let mut q = GfxQueue::new(1, QueueInfo::default());
        q.submit_commands(5, 10, 100);
        q.reset_stats();
        assert_eq!(q.num_draw_calls, 0);
        assert_eq!(q.num_instances, 0);
        assert_eq!(q.num_tris, 0);
    }

    #[test]
    fn test_queue_types() {
        let compute_q = GfxQueue::new(2, QueueInfo { queue_type: QueueType::Compute });
        assert_eq!(compute_q.get_type(), QueueType::Compute);

        let transfer_q = GfxQueue::new(3, QueueInfo { queue_type: QueueType::Transfer });
        assert_eq!(transfer_q.get_type(), QueueType::Transfer);
    }
}
