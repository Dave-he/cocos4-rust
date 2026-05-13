/****************************************************************************
Rust port of Cocos Creator ForwardPipeline
Original C++ version Copyright (c) 2020-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/
// SPDX-License-Identifier: MIT

use super::super::render_pipeline::{RenderPipeline, RenderPipelineInfo};

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
        self.base.activate();
        true
    }

    pub fn render(&mut self, cameras: &[u64]) {
        for _camera_id in cameras {
        }
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
    use super::*;
    use super::super::super::render_pipeline::RenderPipelineInfo;

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
