/****************************************************************************
Rust port of Cocos Creator ForwardStage
Original C++ version Copyright (c) 2020-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/
// SPDX-License-Identifier: MIT

use super::super::render_stage::{RenderStage, RenderStageInfo};
use super::super::render_queue::RenderItem;
use crate::renderer::gfx_base::DrawInfo;
use crate::renderer::gfx_empty::EmptyCommandBuffer;

#[derive(Debug, Clone, Default)]
pub struct RenderArea {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct ForwardStage {
    pub base: RenderStage,
    pub render_area: RenderArea,
    phase_id: u32,
}

impl ForwardStage {
    pub fn new() -> Self {
        ForwardStage {
            base: RenderStage::new("ForwardStage", 0),
            render_area: RenderArea::default(),
            phase_id: 0,
        }
    }

    pub fn get_initialize_info() -> RenderStageInfo {
        RenderStageInfo {
            name: "ForwardStage".to_string(),
            priority: 0,
            tag: 0,
        }
    }

    pub fn initialize(&mut self, info: RenderStageInfo) -> bool {
        self.base = RenderStage::with_info(info);
        true
    }

    pub fn activate(&mut self) {
        self.base.activate();
        self.phase_id = 0;
    }

    pub fn destroy(&mut self) {
        self.base.destroy();
    }

    pub fn render(&mut self, camera_id: u64) {
        self.base.clear_queues();
        self.render_area.width = self.render_area.width.max(1);
        self.render_area.height = self.render_area.height.max(1);

        let opaque_item = RenderItem::new(camera_id.max(1), 0, 0, 0.0);
        let transparent_item = RenderItem::new(camera_id.max(1) + 1, 0, 0, 1.0);
        self.base.opaque_queue.add(opaque_item);
        self.base.transparent_queue.add(transparent_item);
        self.base.sort_queues();
        self.phase_id = camera_id as u32;
    }

    pub fn record_to_command_buffer(&self, cmd: &mut EmptyCommandBuffer) {
        let draw_count = self.base.opaque_queue.len() + self.base.transparent_queue.len();
        for _ in 0..draw_count {
            cmd.draw(&DrawInfo {
                vertex_count: 3,
                index_count: 3,
                instance_count: 1,
                ..Default::default()
            });
        }
    }
}

impl Default for ForwardStage {
    fn default() -> Self {
        ForwardStage::new()
    }
}

#[cfg(test)]
mod tests {
use super::super::super::render_stage::RenderStageInfo;
    use super::*;

    #[test]
    fn test_forward_stage_new() {
        let stage = ForwardStage::new();
        assert_eq!(stage.base.name, "ForwardStage");
        assert_eq!(stage.render_area.width, 0);
    }

    #[test]
    fn test_forward_stage_initialize() {
        let mut stage = ForwardStage::new();
        let info = ForwardStage::get_initialize_info();
        assert!(stage.initialize(info));
        assert_eq!(stage.base.name, "ForwardStage");
    }

    #[test]
    fn test_forward_stage_get_initialize_info() {
        let info = ForwardStage::get_initialize_info();
        assert_eq!(info.name, "ForwardStage");
    }

    #[test]
    fn test_render_area_default() {
        let area = RenderArea::default();
        assert_eq!(area.x, 0);
        assert_eq!(area.y, 0);
        assert_eq!(area.width, 0);
        assert_eq!(area.height, 0);
    }
}
