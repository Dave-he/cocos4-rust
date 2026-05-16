/****************************************************************************
Rust port of Cocos Creator ForwardPipeline
Original C++ version Copyright (c) 2020-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/
// SPDX-License-Identifier: MIT

use super::super::render_pipeline::{RenderPipeline, RenderPipelineInfo};
use super::forward_flow::ForwardFlow;
use super::forward_stage::ForwardStage;
use crate::renderer::gfx_base::CommandBufferInfo;
use crate::renderer::gfx_empty::EmptyCommandBuffer;

#[derive(Debug)]
pub struct ForwardPipeline {
    pub base: RenderPipeline,
    valid_lights: Vec<u64>,
    light_buffers: Vec<u64>,
    light_index_offsets: Vec<u32>,
    light_indices: Vec<u32>,
}

impl ForwardPipeline {
    pub fn new() -> Self {
        ForwardPipeline {
            base: RenderPipeline::new(),
            valid_lights: Vec::new(),
            light_buffers: Vec::new(),
            light_index_offsets: Vec::new(),
            light_indices: Vec::new(),
        }
    }

    pub fn initialize(&mut self, info: RenderPipelineInfo) -> bool {
        self.base.initialize(info);
        true
    }

    pub fn destroy(&mut self) -> bool {
        self.valid_lights.clear();
        self.light_buffers.clear();
        self.light_index_offsets.clear();
        self.light_indices.clear();
        self.base.destroy();
        self.base.initialized = false;
        true
    }

    pub fn activate(&mut self) -> bool {
        if self.base.flows.is_empty() {
            let mut flow = ForwardFlow::new();
            let mut stage = ForwardStage::new();
            stage.activate();
            flow.base.add_stage(stage.base);
            self.base.add_flow(flow.base);
        }
        self.base.activate();
        true
    }

    pub fn render(&mut self, cameras: &[u64]) {
let mut command_buffer = EmptyCommandBuffer::new(CommandBufferInfo::default());
        command_buffer.begin();

        let flow_count = self.base.flows.len();
        let mut stage_count = 0usize;
        let mut queue_count = 0usize;

        for camera_id in cameras {
            for flow in &mut self.base.flows {
                if !flow.enabled {
                    continue;
                }
                for stage in &mut flow.stages {
                    if !stage.enabled {
                        continue;
                    }
                    let mut forward_stage = ForwardStage::new();
                    forward_stage.base = std::mem::take(stage);
                    forward_stage.render(*camera_id);
                    queue_count += forward_stage.base.opaque_queue.len();
                    queue_count += forward_stage.base.transparent_queue.len();
                    forward_stage.record_to_command_buffer(&mut command_buffer);
                    *stage = forward_stage.base;
                    stage_count += 1;
                }
            }
        }

        command_buffer.end();
        self.base.render_data.last_rendered_camera_count = cameras.len();
        self.base.render_data.last_rendered_flow_count = flow_count;
        self.base.render_data.last_rendered_stage_count = stage_count;
        self.base.render_data.last_render_queue_count = queue_count;
    }

    pub fn get_valid_lights(&self) -> &[u64] {
        &self.valid_lights
    }

    pub fn get_light_buffers(&self) -> &[u64] {
        &self.light_buffers
    }

    pub fn get_light_index_offsets(&self) -> &[u32] {
        &self.light_index_offsets
    }
}

impl Default for ForwardPipeline {
    fn default() -> Self {
        ForwardPipeline::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::render_pipeline::RenderPipelineInfo;
    use super::*;

    #[test]
    fn test_forward_pipeline_new() {
        let pipeline = ForwardPipeline::new();
        assert!(!pipeline.base.initialized);
        assert!(pipeline.valid_lights.is_empty());
    }

    #[test]
    fn test_forward_pipeline_initialize() {
        let mut pipeline = ForwardPipeline::new();
        let info = RenderPipelineInfo {
            name: "ForwardPipeline".to_string(),
            flows: Vec::new(),
            tag: 1,
        };
        assert!(pipeline.initialize(info));
        assert!(pipeline.base.initialized);
        assert_eq!(pipeline.base.name, "ForwardPipeline");
    }

    #[test]
    fn test_forward_pipeline_destroy() {
        let mut pipeline = ForwardPipeline::new();
        let info = RenderPipelineInfo {
            name: "ForwardPipeline".to_string(),
            flows: Vec::new(),
            tag: 0,
        };
        pipeline.initialize(info);
        assert!(pipeline.destroy());
        assert!(!pipeline.base.initialized);
    }

    #[test]
    fn test_forward_pipeline_default() {
        let pipeline = ForwardPipeline::default();
        assert!(pipeline.valid_lights.is_empty());
    }
}
