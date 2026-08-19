/****************************************************************************
Rust port of Cocos Creator Render Pipeline
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::defines::MacroRecord;
use super::pipeline_scene_data::PipelineSceneData;
use super::render_flow::RenderFlow;
use super::shadow::ShadowsInfo;
use super::states::PipelineStateManager;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct RenderPipelineInfo {
    pub name: String,
    pub flows: Vec<RenderFlow>,
    pub tag: u32,
}

#[derive(Debug, Clone, Default)]
pub struct BloomRenderData {
    pub prefiter_tex_id: u64,
    pub downsample_tex_ids: Vec<u64>,
    pub upsample_tex_ids: Vec<u64>,
    pub combine_tex_id: u64,
    pub prefilter_framebuffer_id: u64,
    pub downsample_framebuffer_ids: Vec<u64>,
    pub upsample_framebuffer_ids: Vec<u64>,
    pub combine_framebuffer_id: u64,
    pub render_pass_id: u64,
    pub sampler_id: u64,
}

#[derive(Debug, Clone, Default)]
pub struct PipelineRenderData {
    pub output_framebuffer_id: u64,
    pub output_render_target_ids: Vec<u64>,
    pub output_depth_id: u64,
    pub sampler_id: u64,
    pub bloom: Option<BloomRenderData>,
    pub last_rendered_camera_count: usize,
    pub last_rendered_flow_count: usize,
    pub last_rendered_stage_count: usize,
    pub last_render_queue_count: usize,
}

#[derive(Debug)]
pub struct RenderPipeline {
    pub name: String,
    pub tag: u32,
    pub flows: Vec<RenderFlow>,
    pub shadows: ShadowsInfo,
    pub state_manager: PipelineStateManager,
    pub scene_data: PipelineSceneData,
    pub render_data: PipelineRenderData,
    pub macros: MacroRecord,
    pub constant_macros: String,
    pub initialized: bool,
    pub width: u32,
    pub height: u32,
    pub cluster_enabled: bool,
    pub bloom_enabled: bool,
    pub render_passes: HashMap<u64, u64>,
}

impl RenderPipeline {
    pub fn new() -> Self {
        RenderPipeline {
            name: String::new(),
            tag: 0,
            flows: Vec::new(),
            shadows: ShadowsInfo::default(),
            state_manager: PipelineStateManager::new(),
            scene_data: PipelineSceneData::new(),
            render_data: PipelineRenderData::default(),
            macros: MacroRecord::default(),
            constant_macros: String::new(),
            initialized: false,
            width: 0,
            height: 0,
            cluster_enabled: false,
            bloom_enabled: false,
            render_passes: HashMap::new(),
        }
    }

    pub fn initialize(&mut self, info: RenderPipelineInfo) -> bool {
        self.name = info.name;
        self.flows = info.flows;
        if info.tag != 0 {
            self.tag = info.tag;
        }
        self.initialized = true;
        true
    }

    pub fn destroy(&mut self) {
        for flow in &mut self.flows {
            flow.destroy();
        }
        self.flows.clear();
        self.render_passes.clear();
        self.initialized = false;
    }

    pub fn activate(&mut self) {
        self.scene_data.activate();
        self.macros.value = String::new();
        for flow in &mut self.flows {
            flow.activate();
        }
    }

    pub fn on_global_pipeline_state_changed(&mut self) {
        self.state_manager.set_shadow_enabled(self.shadows.enabled);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn add_flow(&mut self, flow: RenderFlow) {
        let mut flows = std::mem::take(&mut self.flows);
        flows.push(flow);
        flows.sort_by_key(|f| f.priority);
        self.flows = flows;
    }

    pub fn remove_flow(&mut self, name: &str) {
        self.flows.retain(|f| f.name != name);
    }

    pub fn get_flow(&self, name: &str) -> Option<&RenderFlow> {
        self.flows.iter().find(|f| f.name == name)
    }

    pub fn get_flow_mut(&mut self, name: &str) -> Option<&mut RenderFlow> {
        self.flows.iter_mut().find(|f| f.name == name)
    }

    pub fn get_flows(&self) -> &[RenderFlow] {
        &self.flows
    }

    pub fn get_shadows(&self) -> &ShadowsInfo {
        &self.shadows
    }

    pub fn get_shadows_mut(&mut self) -> &mut ShadowsInfo {
        &mut self.shadows
    }

    pub fn get_scene_data(&self) -> &PipelineSceneData {
        &self.scene_data
    }

    pub fn get_scene_data_mut(&mut self) -> &mut PipelineSceneData {
        &mut self.scene_data
    }

    pub fn get_state_manager(&self) -> &PipelineStateManager {
        &self.state_manager
    }

    pub fn get_state_manager_mut(&mut self) -> &mut PipelineStateManager {
        &mut self.state_manager
    }

    pub fn get_render_data(&self) -> &PipelineRenderData {
        &self.render_data
    }

    pub fn get_render_data_mut(&mut self) -> &mut PipelineRenderData {
        &mut self.render_data
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn get_tag(&self) -> u32 {
        self.tag
    }

    pub fn set_tag(&mut self, tag: u32) {
        self.tag = tag;
    }

    pub fn get_shading_scale(&self) -> f32 {
        self.scene_data.get_shading_scale()
    }

    pub fn set_shading_scale(&mut self, val: f32) {
        self.scene_data.set_shading_scale(val);
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn is_hdr(&self) -> bool {
        self.scene_data.is_hdr()
    }

    pub fn set_hdr(&mut self, val: bool) {
        self.scene_data.set_hdr(val);
    }

    pub fn set_cluster_enabled(&mut self, val: bool) {
        self.cluster_enabled = val;
    }

    pub fn is_cluster_enabled(&self) -> bool {
        self.cluster_enabled
    }

    pub fn set_bloom_enabled(&mut self, val: bool) {
        self.bloom_enabled = val;
    }

    pub fn is_bloom_enabled(&self) -> bool {
        self.bloom_enabled
    }

    pub fn get_constant_macros(&self) -> &str {
        &self.constant_macros
    }

    pub fn set_macro_string(&mut self, name: &str, value: &str) {
        self.macros.name = name.to_string();
        self.macros.value = value.to_string();
    }

    pub fn get_macro_string(&self) -> &str {
        &self.macros.value
    }
}

impl Default for RenderPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_pipeline_new() {
        let pipeline = RenderPipeline::new();
        assert!(!pipeline.is_initialized());
        assert!(pipeline.flows.is_empty());
        assert!(!pipeline.is_cluster_enabled());
        assert!(!pipeline.is_bloom_enabled());
    }

    #[test]
    fn test_render_pipeline_initialize() {
        let mut pipeline = RenderPipeline::new();
        let result = pipeline.initialize(RenderPipelineInfo {
            name: "forward".to_string(),
            flows: Vec::new(),
            tag: 0,
        });
        assert!(result);
        assert!(pipeline.is_initialized());
        assert_eq!(pipeline.name, "forward");
    }

    #[test]
    fn test_render_pipeline_add_flow() {
        let mut pipeline = RenderPipeline::new();
        pipeline.add_flow(RenderFlow::new("shadows", 10));
        pipeline.add_flow(RenderFlow::new("forward", 0));
        assert_eq!(pipeline.flows.len(), 2);
        assert_eq!(pipeline.flows[0].name, "forward");
        assert_eq!(pipeline.flows[1].name, "shadows");
    }

    #[test]
    fn test_render_pipeline_resize() {
        let mut pipeline = RenderPipeline::new();
        pipeline.resize(1920, 1080);
        assert_eq!(pipeline.get_width(), 1920);
        assert_eq!(pipeline.get_height(), 1080);
    }

    #[test]
    fn test_render_pipeline_shadows() {
        let mut pipeline = RenderPipeline::new();
        pipeline.get_shadows_mut().set_enabled(true);
        assert!(pipeline.get_shadows().enabled);
    }

    #[test]
    fn test_render_pipeline_hdr() {
        let mut pipeline = RenderPipeline::new();
        pipeline.set_hdr(true);
        assert!(pipeline.is_hdr());
    }

    #[test]
    fn test_render_pipeline_cluster() {
        let mut pipeline = RenderPipeline::new();
        pipeline.set_cluster_enabled(true);
        assert!(pipeline.is_cluster_enabled());
    }

    #[test]
    fn test_render_pipeline_bloom() {
        let mut pipeline = RenderPipeline::new();
        pipeline.set_bloom_enabled(true);
        assert!(pipeline.is_bloom_enabled());
    }

    #[test]
    fn test_render_pipeline_remove_flow() {
        let mut pipeline = RenderPipeline::new();
        pipeline.add_flow(RenderFlow::new("forward", 0));
        pipeline.add_flow(RenderFlow::new("shadow", 1));
        pipeline.remove_flow("shadow");
        assert_eq!(pipeline.get_flows().len(), 1);
        assert!(pipeline.get_flow("shadow").is_none());
    }
}
