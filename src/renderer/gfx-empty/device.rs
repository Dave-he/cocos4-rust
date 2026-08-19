/****************************************************************************
Rust port of Cocos Creator GFX Empty Device
Null/no-op backend for testing and CI.
****************************************************************************/

use crate::renderer::gfx_base::{
    BufferInfo, BufferViewInfo, CommandBufferInfo, DescriptorSetLayoutInfo, DeviceInfo,
    FormatFeature, FramebufferInfo, GfxBuffer, GfxCommandBuffer, GfxDescriptorSet,
    GfxDescriptorSetLayout, GfxDevice, GfxFramebuffer, GfxInputAssembler, GfxPipelineLayout,
    GfxPipelineState, GfxQueryPool, GfxQueue, GfxRenderPass, GfxSampler, GfxShader, GfxSwapchain,
    GfxTexture, InputAssemblerInfo, MemoryStatus, PipelineLayoutInfo, PipelineStateInfo,
    QueryPoolInfo, QueueInfo, RenderPassInfo, SamplerInfo, ShaderInfo, SwapchainInfo, TextureInfo,
    TextureViewInfo, API,
};

use super::EmptyCommandBuffer;

pub struct EmptyDevice {
    device: GfxDevice,
    queue: Option<GfxQueue>,
    cmd_buff: Option<EmptyCommandBuffer>,
}

impl EmptyDevice {
    pub fn new(info: DeviceInfo) -> Self {
        let mut device = GfxDevice::new(info);
        device.api = API::Unknown;
        device.device_name = "Empty Device".to_string();
        device.renderer = "Null Renderer".to_string();
        device.vendor = "Cocos4-Rust".to_string();
        Self {
            device,
            queue: None,
            cmd_buff: None,
        }
    }

    pub fn initialize(&mut self) -> bool {
        self.device.api = API::Unknown;
        let all_features = FormatFeature::RENDER_TARGET
            | FormatFeature::SAMPLED_TEXTURE
            | FormatFeature::LINEAR_FILTER
            | FormatFeature::STORAGE_TEXTURE
            | FormatFeature::VERTEX_ATTRIBUTE;
        self.device.set_all_format_features(all_features, true);
        let q = self.device.create_queue(QueueInfo::default());
        self.queue = Some(q);
        let cmd = EmptyCommandBuffer::new(CommandBufferInfo::default());
        self.cmd_buff = Some(cmd);
        true
    }

    pub fn acquire(&mut self) {}

    pub fn present(&mut self) {
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    pub fn frame_sync(&mut self) {}

    pub fn flush_commands(&mut self, _cmd_buffs: &[&GfxCommandBuffer]) {}

    pub fn create_command_buffer(&mut self, info: CommandBufferInfo) -> EmptyCommandBuffer {
        EmptyCommandBuffer::new(info)
    }

    pub fn create_queue(&mut self, info: QueueInfo) -> GfxQueue {
        self.device.create_queue(info)
    }

    pub fn create_swapchain(&mut self, info: SwapchainInfo) -> GfxSwapchain {
        self.device.create_swapchain(info)
    }

    pub fn create_buffer(&mut self, info: BufferInfo) -> GfxBuffer {
        self.device.create_buffer(info)
    }

    pub fn create_buffer_view(&mut self, info: BufferViewInfo) -> GfxBuffer {
        self.device.create_buffer_view(info)
    }

    pub fn create_texture(&mut self, info: TextureInfo) -> GfxTexture {
        self.device.create_texture(info)
    }

    pub fn create_texture_view(&mut self, info: TextureViewInfo) -> GfxTexture {
        self.device.create_texture_view(info)
    }

    pub fn create_shader(&mut self, info: ShaderInfo) -> GfxShader {
        self.device.create_shader(info)
    }

    pub fn create_input_assembler(&mut self, info: InputAssemblerInfo) -> GfxInputAssembler {
        self.device.create_input_assembler(info)
    }

    pub fn create_render_pass(&mut self, info: RenderPassInfo) -> GfxRenderPass {
        self.device.create_render_pass(info)
    }

    pub fn create_framebuffer(
        &mut self,
        info: FramebufferInfo,
        width: u32,
        height: u32,
    ) -> GfxFramebuffer {
        self.device.create_framebuffer(info, width, height)
    }

    pub fn create_descriptor_set_layout(
        &mut self,
        info: DescriptorSetLayoutInfo,
    ) -> GfxDescriptorSetLayout {
        self.device.create_descriptor_set_layout(info)
    }

    pub fn create_descriptor_set(&mut self, layout_id: u32) -> GfxDescriptorSet {
        self.device.create_descriptor_set(layout_id)
    }

    pub fn create_pipeline_layout(&mut self, info: PipelineLayoutInfo) -> GfxPipelineLayout {
        self.device.create_pipeline_layout(info)
    }

    pub fn create_pipeline_state(&mut self, info: PipelineStateInfo) -> GfxPipelineState {
        self.device.create_pipeline_state(info)
    }

    pub fn create_sampler(&mut self, info: SamplerInfo) -> GfxSampler {
        self.device.create_sampler(info)
    }

    pub fn create_query_pool(&mut self, info: QueryPoolInfo) -> GfxQueryPool {
        self.device.create_query_pool(info)
    }

    pub fn get_device(&self) -> &GfxDevice {
        &self.device
    }

    pub fn get_device_mut(&mut self) -> &mut GfxDevice {
        &mut self.device
    }

    pub fn get_queue(&self) -> Option<&GfxQueue> {
        self.queue.as_ref()
    }

    pub fn get_command_buffer(&self) -> Option<&EmptyCommandBuffer> {
        self.cmd_buff.as_ref()
    }

    pub fn get_command_buffer_mut(&mut self) -> Option<&mut EmptyCommandBuffer> {
        self.cmd_buff.as_mut()
    }

    pub fn get_memory_status(&self) -> MemoryStatus {
        self.device.get_memory_status()
    }

    pub fn destroy(&mut self) {
        self.queue = None;
        self.cmd_buff = None;
        self.device.destroy();
    }
}

impl Default for EmptyDevice {
    fn default() -> Self {
        Self::new(DeviceInfo::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::gfx_base::*;

    #[test]
    fn test_empty_device_new() {
        let device = EmptyDevice::default();
        assert_eq!(device.get_device().api, API::Unknown);
        assert_eq!(device.get_device().device_name, "Empty Device");
        assert_eq!(device.get_device().renderer, "Null Renderer");
    }

    #[test]
    fn test_empty_device_initialize() {
        let mut device = EmptyDevice::default();
        assert!(device.initialize());
        assert!(device.get_queue().is_some());
        assert!(device.get_command_buffer().is_some());
        let features = device.get_device().get_format_features(Format::RGBA8);
        assert!(features.contains(FormatFeature::RENDER_TARGET));
        assert!(features.contains(FormatFeature::SAMPLED_TEXTURE));
    }

    #[test]
    fn test_empty_device_format_unknown() {
        let mut device = EmptyDevice::default();
        device.initialize();
        let features = device.get_device().get_format_features(Format::Unknown);
        assert_eq!(features, FormatFeature::NONE);
    }

    #[test]
    fn test_empty_device_create_buffer() {
        let mut device = EmptyDevice::default();
        device.initialize();
        let buf = device.create_buffer(BufferInfo {
            size: 256,
            stride: 4,
            ..Default::default()
        });
        assert_eq!(buf.get_size(), 256);
    }

    #[test]
    fn test_empty_device_create_texture() {
        let mut device = EmptyDevice::default();
        device.initialize();
        let tex = device.create_texture(TextureInfo {
            width: 512,
            height: 512,
            ..Default::default()
        });
        assert_eq!(tex.get_width(), 512);
    }

    #[test]
    fn test_empty_device_create_shader() {
        let mut device = EmptyDevice::default();
        device.initialize();
        let shader = device.create_shader(ShaderInfo {
            name: "TestShader".to_string(),
            ..Default::default()
        });
        assert_eq!(shader.get_name(), "TestShader");
    }

    #[test]
    fn test_empty_device_create_command_buffer() {
        let mut device = EmptyDevice::default();
        device.initialize();
        let mut cmd = device.create_command_buffer(CommandBufferInfo::default());
        cmd.begin();
        cmd.draw(&DrawInfo {
            index_count: 6,
            instance_count: 1,
            ..Default::default()
        });
        cmd.end();
        assert_eq!(cmd.get_num_draw_calls(), 1);
    }

    #[test]
    fn test_empty_device_create_queue() {
        let mut device = EmptyDevice::default();
        device.initialize();
        let queue = device.create_queue(QueueInfo::default());
        assert!(queue.id > 0);
    }

    #[test]
    fn test_empty_device_create_sampler() {
        let mut device = EmptyDevice::default();
        device.initialize();
        let sampler = device.create_sampler(SamplerInfo::default());
        assert!(sampler.id > 0);
    }

    #[test]
    fn test_empty_device_create_render_pass() {
        let mut device = EmptyDevice::default();
        device.initialize();
        let rp = device.create_render_pass(RenderPassInfo::default());
        assert!(rp.id > 0);
    }

    #[test]
    fn test_empty_device_create_pipeline_state() {
        let mut device = EmptyDevice::default();
        device.initialize();
        let pso = device.create_pipeline_state(PipelineStateInfo::default());
        assert!(pso.id > 0);
    }

    #[test]
    fn test_empty_device_create_descriptor_set() {
        let mut device = EmptyDevice::default();
        device.initialize();
        let layout = device.create_descriptor_set_layout(DescriptorSetLayoutInfo::default());
        let ds = device.create_descriptor_set(layout.id);
        assert!(ds.id > 0);
        assert_eq!(ds.layout_id, layout.id);
    }

    #[test]
    fn test_empty_device_acquire_present() {
        let mut device = EmptyDevice::default();
        device.initialize();
        device.acquire();
        device.present();
    }

    #[test]
    fn test_empty_device_flush_commands() {
        let mut device = EmptyDevice::default();
        device.initialize();
        device.flush_commands(&[]);
    }

    #[test]
    fn test_empty_device_destroy() {
        let mut device = EmptyDevice::default();
        device.initialize();
        device.destroy();
        assert!(device.get_queue().is_none());
        assert!(device.get_command_buffer().is_none());
    }

    #[test]
    fn test_empty_device_draw_through_command_buffer() {
        let mut empty = EmptyDevice::default();
        empty.initialize();
        let mut cmd = empty.create_command_buffer(CommandBufferInfo::default());
        cmd.begin();
        cmd.draw(&DrawInfo {
            index_count: 6,
            instance_count: 1,
            ..Default::default()
        });
        cmd.end();
        assert_eq!(cmd.get_num_draw_calls(), 1);
        assert_eq!(cmd.get_num_tris(), 2);
    }

    #[test]
    fn test_empty_device_caps() {
        let device = EmptyDevice::default();
        let caps = device.get_device().get_capabilities();
        assert_eq!(caps.ubo_offset_alignment, 1);
    }

    #[test]
    fn test_empty_device_memory_status() {
        let device = EmptyDevice::default();
        assert_eq!(device.get_memory_status(), MemoryStatus::Medium);
    }

    #[test]
    fn test_empty_device_empty_cmd_buffer_noop() {
        let mut empty = EmptyDevice::default();
        empty.initialize();
        let mut cmd = empty.create_command_buffer(CommandBufferInfo::default());
        cmd.begin();
        cmd.set_viewport(&Viewport::new(0, 0, 800, 600));
        cmd.set_scissor(&Rect::default());
        cmd.bind_pipeline_state(1);
        cmd.bind_descriptor_set(0, 1, &[]);
        cmd.bind_input_assembler(1);
        cmd.end();
    }

    #[test]
    fn test_empty_device_default_cmd_buff() {
        let mut empty = EmptyDevice::default();
        empty.initialize();
        let cmd = empty.get_command_buffer_mut().unwrap();
        cmd.begin();
        cmd.draw(&DrawInfo {
            index_count: 3,
            instance_count: 1,
            ..Default::default()
        });
        cmd.end();
        assert_eq!(cmd.get_num_draw_calls(), 1);
    }
}
