/****************************************************************************
Rust port of Cocos Creator Render Stage
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::defines::RenderPassStage;
use super::render_queue::RenderQueue;

#[derive(Debug)]
pub struct RenderStageInfo {
    pub name: String,
    pub priority: u32,
    pub tag: u32,
}

impl Default for RenderStageInfo {
    fn default() -> Self {
        RenderStageInfo {
            name: String::new(),
            priority: 0,
            tag: 0,
        }
    }
}

#[derive(Debug)]
pub struct RenderStage {
    pub name: String,
    pub priority: u32,
    pub tag: u32,
    pub enabled: bool,
    pub pass_stage: RenderPassStage,
    pub opaque_queue: RenderQueue,
    pub transparent_queue: RenderQueue,
}

impl RenderStage {
    pub fn new(name: &str, priority: u32) -> Self {
        RenderStage {
            name: name.to_string(),
            priority,
            tag: 0,
            enabled: true,
            pass_stage: RenderPassStage::DEFAULT,
            opaque_queue: RenderQueue::new(false),
            transparent_queue: RenderQueue::new(true),
        }
    }

    pub fn with_info(info: RenderStageInfo) -> Self {
        RenderStage {
            name: info.name,
            priority: info.priority,
            tag: info.tag,
            enabled: true,
            pass_stage: RenderPassStage::DEFAULT,
            opaque_queue: RenderQueue::new(false),
            transparent_queue: RenderQueue::new(true),
        }
    }

    pub fn initialize(&mut self, info: RenderStageInfo) {
        self.name = info.name;
        self.priority = info.priority;
        self.tag = info.tag;
    }

    pub fn activate(&mut self) {
        self.enabled = true;
        self.opaque_queue.clear();
        self.transparent_queue.clear();
    }

    pub fn destroy(&mut self) {
        self.enabled = false;
        self.opaque_queue.clear();
        self.transparent_queue.clear();
    }

    pub fn clear_queues(&mut self) {
        self.opaque_queue.clear();
        self.transparent_queue.clear();
    }

    pub fn sort_queues(&mut self) {
        self.opaque_queue.sort();
        self.transparent_queue.sort();
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_priority(&self) -> u32 {
        self.priority
    }

    pub fn set_priority(&mut self, priority: u32) {
        self.priority = priority;
    }

    pub fn get_tag(&self) -> u32 {
        self.tag
    }

    pub fn set_tag(&mut self, tag: u32) {
        self.tag = tag;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn get_pass_stage(&self) -> RenderPassStage {
        self.pass_stage
    }

    pub fn set_pass_stage(&mut self, stage: RenderPassStage) {
        self.pass_stage = stage;
    }

    pub fn get_opaque_queue(&self) -> &RenderQueue {
        &self.opaque_queue
    }

    pub fn get_opaque_queue_mut(&mut self) -> &mut RenderQueue {
        &mut self.opaque_queue
    }

    pub fn get_transparent_queue(&self) -> &RenderQueue {
        &self.transparent_queue
    }

    pub fn get_transparent_queue_mut(&mut self) -> &mut RenderQueue {
        &mut self.transparent_queue
    }
}

impl Default for RenderStage {
    fn default() -> Self {
        Self::new("default", 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_stage_new() {
        let stage = RenderStage::new("shadow", 1);
        assert_eq!(stage.name, "shadow");
        assert_eq!(stage.priority, 1);
        assert!(stage.enabled);
        assert_eq!(stage.get_pass_stage(), RenderPassStage::DEFAULT);
    }

    #[test]
    fn test_render_stage_destroy() {
        let mut stage = RenderStage::new("test", 0);
        stage.destroy();
        assert!(!stage.enabled);
    }

    #[test]
    fn test_render_stage_enable_disable() {
        let mut stage = RenderStage::new("test", 0);
        stage.set_enabled(false);
        assert!(!stage.is_enabled());
    }

    #[test]
    fn test_render_stage_tag() {
        let mut stage = RenderStage::new("test", 0);
        stage.set_tag(5);
        assert_eq!(stage.get_tag(), 5);
    }

    #[test]
    fn test_render_stage_pass_stage() {
        let mut stage = RenderStage::new("test", 0);
        stage.set_pass_stage(RenderPassStage::UI);
        assert_eq!(stage.get_pass_stage(), RenderPassStage::UI);
    }
}
