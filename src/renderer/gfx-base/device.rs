/****************************************************************************
Rust port of Cocos Creator GFX Device
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::{
    API, BufferInfo, BufferViewInfo, CommandBufferInfo, DescriptorSetLayoutInfo, DeviceCaps,
    Feature, Format, FormatFeature, FramebufferInfo, GfxBuffer, GfxCommandBuffer,
    GfxDescriptorSet, GfxDescriptorSetLayout, GfxFramebuffer, GfxInputAssembler,
    GfxPipelineLayout, GfxPipelineState, GfxQueryPool, GfxQueue, GfxRenderPass, GfxSampler,
    GfxShader, GfxSwapchain, GfxTexture, InputAssemblerInfo, PipelineLayoutInfo,
    PipelineStateInfo, QueryPoolInfo, QueueInfo, RenderPassInfo, SamplerInfo, ShaderInfo,
    SwapchainInfo, TextureInfo, TextureViewInfo,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryStatus {
    Low = 0,
    Medium = 1,
    High = 2,
}

#[derive(Debug, Clone)]
pub struct BindingMappingInfo {
    pub max_block_counts: Vec<u32>,
    pub max_sampler_texture_counts: Vec<u32>,
    pub max_sampler_counts: Vec<u32>,
    pub max_texture_counts: Vec<u32>,
    pub max_buffer_counts: Vec<u32>,
    pub max_image_counts: Vec<u32>,
    pub max_subpass_input_counts: Vec<u32>,
    pub set_indices: Vec<u32>,
}

impl Default for BindingMappingInfo {
    fn default() -> Self {
        BindingMappingInfo {
            max_block_counts: vec![0],
            max_sampler_texture_counts: vec![0],
            max_sampler_counts: vec![0],
            max_texture_counts: vec![0],
            max_buffer_counts: vec![0],
            max_image_counts: vec![0],
            max_subpass_input_counts: vec![0],
            set_indices: vec![0],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DeviceInfo {
    pub binding_mapping_info: BindingMappingInfo,
}

#[derive(Debug)]
pub struct GfxDevice {
    pub info: DeviceInfo,
    pub caps: DeviceCaps,
    pub api: API,
    pub device_name: String,
    pub renderer: String,
    pub vendor: String,
    next_id: u32,
    pub num_draw_calls: u32,
    pub num_instances: u32,
    pub num_tris: u32,
    features: [bool; 16],
    format_features: Vec<FormatFeature>,
}

impl GfxDevice {
    pub fn new(info: DeviceInfo) -> Self {
        let format_count = 256;
        GfxDevice {
            info,
            caps: DeviceCaps::new(),
            api: API::Unknown,
            device_name: "Software Device".to_string(),
            renderer: String::new(),
            vendor: String::new(),
            next_id: 1,
            num_draw_calls: 0,
            num_instances: 0,
            num_tris: 0,
            features: [false; 16],
            format_features: vec![FormatFeature::NONE; format_count],
        }
    }

    fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn has_feature(&self, feature: Feature) -> bool {
        let idx = feature as usize;
        if idx < self.features.len() {
            self.features[idx]
        } else {
            false
        }
    }

    pub fn get_format_features(&self, format: Format) -> FormatFeature {
        let idx = format as usize;
        if idx < self.format_features.len() {
            self.format_features[idx]
        } else {
            FormatFeature::NONE
        }
    }

    pub fn get_api(&self) -> API {
        self.api
    }

    pub fn get_device_name(&self) -> &str {
        &self.device_name
    }

    pub fn get_capabilities(&self) -> &DeviceCaps {
        &self.caps
    }

    pub fn create_command_buffer(&mut self, info: CommandBufferInfo) -> GfxCommandBuffer {
        let id = self.next_id();
        GfxCommandBuffer::new(id, info)
    }

    pub fn create_queue(&mut self, info: QueueInfo) -> GfxQueue {
        let id = self.next_id();
        GfxQueue::new(id, info)
    }

    pub fn create_query_pool(&mut self, info: QueryPoolInfo) -> GfxQueryPool {
        let id = self.next_id();
        GfxQueryPool::new(id, info)
    }

    pub fn create_swapchain(&mut self, info: SwapchainInfo) -> GfxSwapchain {
        let id = self.next_id();
        GfxSwapchain::new(id, info)
    }

    pub fn create_buffer(&mut self, info: BufferInfo) -> GfxBuffer {
        let id = self.next_id();
        GfxBuffer::new(id, info)
    }

    pub fn create_buffer_view(&mut self, info: BufferViewInfo) -> GfxBuffer {
        let id = self.next_id();
        GfxBuffer::new_view(id, info)
    }

    pub fn create_texture(&mut self, info: TextureInfo) -> GfxTexture {
        let id = self.next_id();
        GfxTexture::new(id, info)
    }

    pub fn create_texture_view(&mut self, info: TextureViewInfo) -> GfxTexture {
        let id = self.next_id();
        GfxTexture::new_view(id, info)
    }

    pub fn create_shader(&mut self, info: ShaderInfo) -> GfxShader {
        let id = self.next_id();
        GfxShader::new(id, info)
    }

    pub fn create_input_assembler(&mut self, info: InputAssemblerInfo) -> GfxInputAssembler {
        let id = self.next_id();
        GfxInputAssembler::new(id, info)
    }

    pub fn create_render_pass(&mut self, info: RenderPassInfo) -> GfxRenderPass {
        let id = self.next_id();
        GfxRenderPass::new(id, info)
    }

    pub fn create_framebuffer(
        &mut self,
        info: FramebufferInfo,
        width: u32,
        height: u32,
    ) -> GfxFramebuffer {
        let id = self.next_id();
        GfxFramebuffer::new(id, info, width, height)
    }

    pub fn create_descriptor_set_layout(
        &mut self,
        info: DescriptorSetLayoutInfo,
    ) -> GfxDescriptorSetLayout {
        let id = self.next_id();
        GfxDescriptorSetLayout::new(id, info)
    }

    pub fn create_descriptor_set(&mut self, layout_id: u32) -> GfxDescriptorSet {
        let id = self.next_id();
        GfxDescriptorSet::new(id, layout_id)
    }

    pub fn create_pipeline_layout(&mut self, info: PipelineLayoutInfo) -> GfxPipelineLayout {
        let id = self.next_id();
        GfxPipelineLayout::new(id, info)
    }

    pub fn create_pipeline_state(&mut self, info: PipelineStateInfo) -> GfxPipelineState {
        let id = self.next_id();
        GfxPipelineState::new(id, info)
    }

    pub fn create_sampler(&mut self, info: SamplerInfo) -> GfxSampler {
        let id = self.next_id();
        GfxSampler::new(id, info)
    }

    pub fn get_memory_status(&self) -> MemoryStatus {
        MemoryStatus::Medium
    }

    pub fn get_num_draw_calls(&self) -> u32 {
        self.num_draw_calls
    }

    pub fn get_num_instances(&self) -> u32 {
        self.num_instances
    }

    pub fn get_num_tris(&self) -> u32 {
        self.num_tris
    }

    pub fn reset_stats(&mut self) {
        self.num_draw_calls = 0;
        self.num_instances = 0;
        self.num_tris = 0;
    }

    pub fn flush_commands(&mut self, cmd_buffs: &[&GfxCommandBuffer]) {
        for cmd in cmd_buffs {
            self.num_draw_calls += cmd.num_draw_calls;
            self.num_instances += cmd.num_instances;
            self.num_tris += cmd.num_tris;
        }
    }
}

impl Default for GfxDevice {
    fn default() -> Self {
        Self::new(DeviceInfo::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::*;

    #[test]
    fn test_device_new() {
        let device = GfxDevice::default();
        assert_eq!(device.get_num_draw_calls(), 0);
        assert_eq!(device.get_memory_status(), MemoryStatus::Medium);
    }

    #[test]
    fn test_device_create_buffer() {
        let mut device = GfxDevice::default();
        let buf = device.create_buffer(BufferInfo {
            size: 256,
            stride: 12,
            ..Default::default()
        });
        assert_eq!(buf.get_size(), 256);
        assert!(buf.id > 0);
    }

    #[test]
    fn test_device_create_texture() {
        let mut device = GfxDevice::default();
        let tex = device.create_texture(TextureInfo {
            width: 512,
            height: 512,
            ..Default::default()
        });
        assert_eq!(tex.get_width(), 512);
        assert_eq!(tex.get_height(), 512);
        assert!(tex.id > 0);
    }

    #[test]
    fn test_device_create_shader() {
        let mut device = GfxDevice::default();
        let shader = device.create_shader(ShaderInfo {
            name: "Test".to_string(),
            ..Default::default()
        });
        assert_eq!(shader.get_name(), "Test");
        assert!(shader.id > 0);
    }

    #[test]
    fn test_device_create_command_buffer() {
        let mut device = GfxDevice::default();
        let cmd = device.create_command_buffer(CommandBufferInfo::default());
        assert!(cmd.id > 0);
        assert_eq!(cmd.info.buffer_type, CommandBufferType::Primary);
    }

    #[test]
    fn test_device_create_queue() {
        let mut device = GfxDevice::default();
        let q = device.create_queue(QueueInfo::default());
        assert!(q.id > 0);
        assert_eq!(q.get_type(), QueueType::Graphics);
    }

    #[test]
    fn test_device_create_render_pass() {
        let mut device = GfxDevice::default();
        let rp = device.create_render_pass(RenderPassInfo::default());
        assert!(rp.id > 0);
        assert_eq!(rp.get_color_attachment_count(), 1);
    }

    #[test]
    fn test_device_create_sampler() {
        let mut device = GfxDevice::default();
        let s = device.create_sampler(SamplerInfo::default());
        assert!(s.id > 0);
    }

    #[test]
    fn test_device_create_pipeline_state() {
        let mut device = GfxDevice::default();
        let pso = device.create_pipeline_state(PipelineStateInfo::default());
        assert!(pso.id > 0);
    }

    #[test]
    fn test_device_create_descriptor_set_layout() {
        let mut device = GfxDevice::default();
        let layout = device.create_descriptor_set_layout(DescriptorSetLayoutInfo::default());
        assert!(layout.id > 0);
    }

    #[test]
    fn test_device_create_descriptor_set() {
        let mut device = GfxDevice::default();
        let layout = device.create_descriptor_set_layout(DescriptorSetLayoutInfo::default());
        let ds = device.create_descriptor_set(layout.id);
        assert!(ds.id > 0);
        assert_eq!(ds.layout_id, layout.id);
    }

    #[test]
    fn test_device_reset_stats() {
        let mut device = GfxDevice { num_draw_calls: 100, num_tris: 500, ..Default::default() };
        device.reset_stats();
        assert_eq!(device.get_num_draw_calls(), 0);
        assert_eq!(device.get_num_tris(), 0);
    }

    #[test]
    fn test_device_flush_commands() {
        let mut device = GfxDevice::default();
        let mut cmd = device.create_command_buffer(CommandBufferInfo::default());
        cmd.begin();
        cmd.draw(&DrawInfo { index_count: 6, instance_count: 1, ..Default::default() });
        cmd.end();
        device.flush_commands(&[&cmd]);
        assert_eq!(device.get_num_draw_calls(), 1);
        assert_eq!(device.get_num_tris(), 2);
    }

    #[test]
    fn test_device_id_increments() {
        let mut device = GfxDevice::default();
        let b1 = device.create_buffer(BufferInfo::default());
        let b2 = device.create_buffer(BufferInfo::default());
        assert!(b2.id > b1.id);
    }
}
