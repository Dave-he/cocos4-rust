use crate::renderer::gfx_base::{
    API, BufferInfo, BufferViewInfo, CommandBufferInfo, DescriptorSetLayoutInfo,
    DeviceInfo, FormatFeature, FramebufferInfo, GfxBuffer,
    GfxCommandBuffer, GfxDescriptorSet, GfxDescriptorSetLayout, GfxDevice, GfxFramebuffer,
    GfxInputAssembler, GfxPipelineLayout, GfxPipelineState, GfxQueryPool, GfxQueue,
    GfxRenderPass, GfxSampler, GfxShader, GfxSwapchain, GfxTexture, InputAssemblerInfo,
    MemoryStatus, PipelineLayoutInfo, PipelineStateInfo, QueryPoolInfo, QueueInfo,
    RenderPassInfo, SamplerInfo, ShaderInfo, SwapchainInfo, TextureInfo, TextureViewInfo,
};

use super::WgpuCommandBuffer;

pub struct WgpuDevice {
    device: GfxDevice,
    queue: Option<GfxQueue>,
    cmd_buff: Option<WgpuCommandBuffer>,
    active_swapchain: Option<GfxSwapchain>,
    frame_number: u64,
}

impl WgpuDevice {
    pub fn new(info: DeviceInfo) -> Self {
        let mut device = GfxDevice::new(info);
        device.api = API::WebGPU;
        device.device_name = "WebGPU Device".to_string();
        device.renderer = "WebGPU Renderer".to_string();
        device.vendor = "Cocos4-Rust".to_string();
        Self {
            device,
            queue: None,
            cmd_buff: None,
            active_swapchain: None,
            frame_number: 0,
        }
    }

    pub fn default() -> Self {
        Self::new(DeviceInfo::default())
    }

    pub fn initialize(&mut self) -> bool {
        self.device.api = API::WebGPU;
        let all_features = FormatFeature::RENDER_TARGET
            | FormatFeature::SAMPLED_TEXTURE
            | FormatFeature::LINEAR_FILTER
            | FormatFeature::STORAGE_TEXTURE
            | FormatFeature::VERTEX_ATTRIBUTE;
        self.device.set_all_format_features(all_features, true);

        self.device.set_format_features(
            crate::renderer::gfx_base::Format::D24S8,
            FormatFeature::RENDER_TARGET | FormatFeature::SAMPLED_TEXTURE,
        );
        self.device.set_format_features(
            crate::renderer::gfx_base::Format::D32F,
            FormatFeature::RENDER_TARGET | FormatFeature::SAMPLED_TEXTURE,
        );
        self.device.set_format_features(
            crate::renderer::gfx_base::Format::D16,
            FormatFeature::RENDER_TARGET | FormatFeature::SAMPLED_TEXTURE,
        );

        let q = self.device.create_queue(QueueInfo::default());
        self.queue = Some(q);
        let cmd = WgpuCommandBuffer::new(CommandBufferInfo::default());
        self.cmd_buff = Some(cmd);
        true
    }

    pub fn acquire(&mut self, swapchain: &mut GfxSwapchain) {
        self.active_swapchain = Some(GfxSwapchain {
            id: swapchain.id,
            info: swapchain.info.clone(),
            color_texture_id: swapchain.color_texture_id,
            depth_stencil_texture_id: swapchain.depth_stencil_texture_id,
            surface_transform: swapchain.surface_transform,
        });
    }

    pub fn present(&mut self, _swapchain: &mut GfxSwapchain) {
        self.active_swapchain = None;
        self.frame_number += 1;
    }

    pub fn frame_sync(&mut self) {}

    pub fn flush_commands(&mut self, cmd_buffs: &[&GfxCommandBuffer]) {
        for cb in cmd_buffs {
            self.device.num_draw_calls += cb.num_draw_calls;
            self.device.num_instances += cb.num_instances;
            self.device.num_tris += cb.num_tris;
        }
    }

    pub fn create_command_buffer(&mut self, info: CommandBufferInfo) -> WgpuCommandBuffer {
        WgpuCommandBuffer::new(info)
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

    pub fn get_command_buffer(&self) -> Option<&WgpuCommandBuffer> {
        self.cmd_buff.as_ref()
    }

    pub fn get_command_buffer_mut(&mut self) -> Option<&mut WgpuCommandBuffer> {
        self.cmd_buff.as_mut()
    }

    pub fn get_memory_status(&self) -> MemoryStatus {
        self.device.get_memory_status()
    }

    pub fn get_frame_number(&self) -> u64 {
        self.frame_number
    }

    pub fn destroy(&mut self) {
        self.queue = None;
        self.cmd_buff = None;
        self.active_swapchain = None;
        self.device.destroy();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::gfx_base::*;

    #[test]
    fn test_wgpu_device_new() {
        let device = WgpuDevice::default();
        assert_eq!(device.get_device().api, API::WebGPU);
        assert_eq!(device.get_device().device_name, "WebGPU Device");
        assert_eq!(device.get_device().renderer, "WebGPU Renderer");
    }

    #[test]
    fn test_wgpu_device_initialize() {
        let mut device = WgpuDevice::default();
        assert!(device.initialize());
        assert!(device.get_queue().is_some());
        assert!(device.get_command_buffer().is_some());
        let features = device.get_device().get_format_features(Format::RGBA8);
        assert!(features.contains(FormatFeature::RENDER_TARGET));
        assert!(features.contains(FormatFeature::SAMPLED_TEXTURE));
    }

    #[test]
    fn test_wgpu_device_format_features() {
        let mut device = WgpuDevice::default();
        device.initialize();
        let rgba_features = device.get_device().get_format_features(Format::RGBA8);
        assert!(rgba_features.contains(FormatFeature::RENDER_TARGET));
        assert!(rgba_features.contains(FormatFeature::SAMPLED_TEXTURE));
    }

    #[test]
    fn test_wgpu_device_create_buffer() {
        let mut device = WgpuDevice::default();
        device.initialize();
        let buf = device.create_buffer(BufferInfo {
            size: 1024,
            stride: 16,
            ..Default::default()
        });
        assert_eq!(buf.get_size(), 1024);
    }

    #[test]
    fn test_wgpu_device_create_texture() {
        let mut device = WgpuDevice::default();
        device.initialize();
        let tex = device.create_texture(TextureInfo {
            width: 1024,
            height: 768,
            ..Default::default()
        });
        assert_eq!(tex.get_width(), 1024);
        assert_eq!(tex.get_height(), 768);
    }

    #[test]
    fn test_wgpu_device_create_shader() {
        let mut device = WgpuDevice::default();
        device.initialize();
        let shader = device.create_shader(ShaderInfo {
            name: "WgpuShader".to_string(),
            ..Default::default()
        });
        assert_eq!(shader.get_name(), "WgpuShader");
    }

    #[test]
    fn test_wgpu_device_create_command_buffer() {
        let mut device = WgpuDevice::default();
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
        assert_eq!(cmd.get_num_tris(), 2);
    }

    #[test]
    fn test_wgpu_device_create_queue() {
        let mut device = WgpuDevice::default();
        device.initialize();
        let queue = device.create_queue(QueueInfo::default());
        assert!(queue.id > 0);
    }

    #[test]
    fn test_wgpu_device_create_sampler() {
        let mut device = WgpuDevice::default();
        device.initialize();
        let sampler = device.create_sampler(SamplerInfo::default());
        assert!(sampler.id > 0);
    }

    #[test]
    fn test_wgpu_device_create_render_pass() {
        let mut device = WgpuDevice::default();
        device.initialize();
        let rp = device.create_render_pass(RenderPassInfo::default());
        assert!(rp.id > 0);
    }

    #[test]
    fn test_wgpu_device_create_pipeline_state() {
        let mut device = WgpuDevice::default();
        device.initialize();
        let pso = device.create_pipeline_state(PipelineStateInfo::default());
        assert!(pso.id > 0);
    }

    #[test]
    fn test_wgpu_device_create_descriptor_set() {
        let mut device = WgpuDevice::default();
        device.initialize();
        let layout = device.create_descriptor_set_layout(DescriptorSetLayoutInfo::default());
        let ds = device.create_descriptor_set(layout.id);
        assert!(ds.id > 0);
        assert_eq!(ds.layout_id, layout.id);
    }

    #[test]
    fn test_wgpu_device_acquire_present() {
        let mut device = WgpuDevice::default();
        device.initialize();
        let mut sw = device.create_swapchain(SwapchainInfo::default());
        device.acquire(&mut sw);
        device.present(&mut sw);
        assert_eq!(device.get_frame_number(), 1);
    }

    #[test]
    fn test_wgpu_device_flush_commands() {
        let mut device = WgpuDevice::default();
        device.initialize();
        device.flush_commands(&[]);
    }

    #[test]
    fn test_wgpu_device_destroy() {
        let mut device = WgpuDevice::default();
        device.initialize();
        device.destroy();
        assert!(device.get_queue().is_none());
        assert!(device.get_command_buffer().is_none());
    }

    #[test]
    fn test_wgpu_device_caps() {
        let device = WgpuDevice::default();
        let caps = device.get_device().get_capabilities();
        assert_eq!(caps.ubo_offset_alignment, 1);
    }

    #[test]
    fn test_wgpu_device_memory_status() {
        let device = WgpuDevice::default();
        assert_eq!(device.get_memory_status(), MemoryStatus::Medium);
    }

    #[test]
    fn test_wgpu_device_complete_frame_flow() {
        let mut device = WgpuDevice::default();
        device.initialize();
        let mut sw = device.create_swapchain(SwapchainInfo::default());
        device.acquire(&mut sw);
        let mut cmd = device.create_command_buffer(CommandBufferInfo::default());
        cmd.begin();
        let rp = device.create_render_pass(RenderPassInfo::default());
        let fb = device.create_framebuffer(FramebufferInfo::default(), 800, 600);
        cmd.begin_render_pass(rp.id, fb.id, &Rect::default(), &[], 1.0, 0);
        let pso = device.create_pipeline_state(PipelineStateInfo::default());
        cmd.bind_pipeline_state(pso.id);
        cmd.draw(&DrawInfo {
            index_count: 6,
            instance_count: 1,
            ..Default::default()
        });
        cmd.end_render_pass();
        cmd.end();
        device.present(&mut sw);
        assert_eq!(cmd.get_num_draw_calls(), 1);
        assert_eq!(device.get_frame_number(), 1);
    }

    #[test]
    fn test_wgpu_device_create_input_assembler() {
        let mut device = WgpuDevice::default();
        device.initialize();
        let ia = device.create_input_assembler(InputAssemblerInfo::default());
        assert!(ia.id > 0);
    }

    #[test]
    fn test_wgpu_device_create_query_pool() {
        let mut device = WgpuDevice::default();
        device.initialize();
        let qp = device.create_query_pool(QueryPoolInfo::default());
        assert!(qp.id > 0);
    }
}
