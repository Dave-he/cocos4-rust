/****************************************************************************
Rust port of Cocos Creator GFX Device Agent
Wraps GfxDevice with multi-threaded command recording/submission separation.
Commands are enqueued into a MessageQueue and flushed to the GPU thread.

Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::threading::{MessageQueue, ThreadPool};
use crate::renderer::gfx_base::{
    BufferInfo, BufferViewInfo, CommandBufferInfo, DescriptorSetLayoutInfo, DeviceInfo,
    FormatFeature, FramebufferInfo, GfxBuffer, GfxCommandBuffer, GfxDescriptorSet,
    GfxDescriptorSetLayout, GfxDevice, GfxFramebuffer, GfxInputAssembler, GfxPipelineLayout,
    GfxPipelineState, GfxQueryPool, GfxQueue, GfxRenderPass, GfxSampler, GfxShader, GfxSwapchain,
    GfxTexture, InputAssemblerInfo, MemoryStatus, PipelineLayoutInfo, PipelineStateInfo,
    QueryPoolInfo, QueueInfo, RenderPassInfo, SamplerInfo, ShaderInfo, SwapchainInfo, TextureInfo,
    TextureViewInfo, API,
};

use super::command_buffer::CommandBufferAgent;

const MAX_CPU_FRAME_AHEAD: u32 = 1;

pub struct DeviceAgent {
    device: GfxDevice,
    queue: Option<GfxQueue>,
    cmd_buff: Option<CommandBufferAgent>,
    main_message_queue: MessageQueue,
    multithreaded: bool,
    frame_index: u32,
    worker_pool: Option<ThreadPool>,
}

impl DeviceAgent {
    pub fn new(info: DeviceInfo) -> Self {
        let device = GfxDevice::new(info);
        DeviceAgent {
            device,
            queue: None,
            cmd_buff: None,
            main_message_queue: MessageQueue::new(),
            multithreaded: false,
            frame_index: 0,
            worker_pool: None,
        }
    }

    pub fn from_device(device: GfxDevice) -> Self {
        DeviceAgent {
            device,
            queue: None,
            cmd_buff: None,
            main_message_queue: MessageQueue::new(),
            multithreaded: false,
            frame_index: 0,
            worker_pool: None,
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

        let cmd = CommandBufferAgent::new(CommandBufferInfo::default());
        self.cmd_buff = Some(cmd);
        self.frame_index = 0;
        true
    }

    pub fn set_multithreaded(&mut self, enabled: bool) {
        self.multithreaded = enabled;
        if enabled {
            if self.worker_pool.is_none() {
                self.worker_pool = Some(ThreadPool::new(2));
            }
        } else {
            if let Some(pool) = self.worker_pool.take() {
                pool.stop();
            }
            self.main_message_queue.flush_messages();
        }
    }

    pub fn is_multithreaded(&self) -> bool {
        self.multithreaded
    }

    pub fn acquire(&mut self) {
        if self.multithreaded {
            self.main_message_queue.enqueue(|| {});
            self.main_message_queue.flush_messages();
        }
    }

    pub fn present(&mut self) {
        if self.multithreaded {
            self.main_message_queue.enqueue(|| {});
            self.main_message_queue.flush_messages();
            self.frame_index = (self.frame_index + 1) % (MAX_CPU_FRAME_AHEAD + 1);
        } else {
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }

    pub fn frame_sync(&mut self) {
        if self.multithreaded {
            self.main_message_queue.flush_messages();
        }
    }

    pub fn flush_commands(&mut self, cmd_buffs: &mut [CommandBufferAgent]) {
        for cmd in cmd_buffs.iter_mut() {
            cmd.flush_messages();
        }
        self.device.num_draw_calls = 0;
        self.device.num_instances = 0;
        self.device.num_tris = 0;
        for cmd in cmd_buffs.iter() {
            let inner = cmd.get_inner();
            self.device.num_draw_calls += inner.num_draw_calls;
            self.device.num_instances += inner.num_instances;
            self.device.num_tris += inner.num_tris;
        }

        if self.multithreaded {
            let draw_calls = self.device.num_draw_calls;
            let instances = self.device.num_instances;
            let tris = self.device.num_tris;
            self.main_message_queue.enqueue(move || {
                let _ = (draw_calls, instances, tris);
            });
        }
    }

    pub fn create_command_buffer(&mut self, info: CommandBufferInfo) -> CommandBufferAgent {
        let cmd = self.device.create_command_buffer(info);
        CommandBufferAgent::from_command_buffer(cmd)
    }

    pub fn create_command_buffer_raw(&mut self, info: CommandBufferInfo) -> GfxCommandBuffer {
        self.device.create_command_buffer(info)
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

    pub fn get_command_buffer(&self) -> Option<&CommandBufferAgent> {
        self.cmd_buff.as_ref()
    }

    pub fn get_command_buffer_mut(&mut self) -> Option<&mut CommandBufferAgent> {
        self.cmd_buff.as_mut()
    }

    pub fn get_memory_status(&self) -> MemoryStatus {
        self.device.get_memory_status()
    }

    pub fn get_frame_index(&self) -> u32 {
        self.frame_index
    }

    pub fn get_main_message_queue(&self) -> &MessageQueue {
        &self.main_message_queue
    }

    pub fn destroy(&mut self) {
        if let Some(pool) = self.worker_pool.take() {
            pool.stop();
        }
        self.main_message_queue.flush_messages();
        self.queue = None;
        self.cmd_buff = None;
        self.device.destroy();
        self.multithreaded = false;
    }
}

impl Default for DeviceAgent {
    fn default() -> Self {
        Self::new(DeviceInfo::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::gfx_base::*;

    #[test]
    fn test_device_agent_new() {
        let agent = DeviceAgent::default();
        assert_eq!(agent.get_device().api, API::Unknown);
        assert!(!agent.is_multithreaded());
    }

    #[test]
    fn test_device_agent_from_device() {
        let device = GfxDevice::default();
        let agent = DeviceAgent::from_device(device);
        assert_eq!(agent.get_device().api, API::Unknown);
        assert!(!agent.is_multithreaded());
    }

    #[test]
    fn test_device_agent_initialize() {
        let mut agent = DeviceAgent::default();
        assert!(agent.initialize());
        assert!(agent.get_queue().is_some());
        assert!(agent.get_command_buffer().is_some());
        let features = agent.get_device().get_format_features(Format::RGBA8);
        assert!(features.contains(FormatFeature::RENDER_TARGET));
    }

    #[test]
    fn test_device_agent_multithreaded_toggle() {
        let mut agent = DeviceAgent::default();
        agent.initialize();
        assert!(!agent.is_multithreaded());
        agent.set_multithreaded(true);
        assert!(agent.is_multithreaded());
        agent.set_multithreaded(false);
        assert!(!agent.is_multithreaded());
    }

    #[test]
    fn test_device_agent_frame_index() {
        let mut agent = DeviceAgent::default();
        agent.initialize();
        assert_eq!(agent.get_frame_index(), 0);
        agent.set_multithreaded(true);
        agent.present();
        assert_eq!(agent.get_frame_index(), 1);
        agent.present();
        assert_eq!(agent.get_frame_index(), 0);
    }

    #[test]
    fn test_device_agent_create_buffer() {
        let mut agent = DeviceAgent::default();
        agent.initialize();
        let buf = agent.create_buffer(BufferInfo {
            size: 256,
            stride: 4,
            ..Default::default()
        });
        assert_eq!(buf.get_size(), 256);
        assert!(buf.id > 0);
    }

    #[test]
    fn test_device_agent_create_texture() {
        let mut agent = DeviceAgent::default();
        agent.initialize();
        let tex = agent.create_texture(TextureInfo {
            width: 512,
            height: 512,
            ..Default::default()
        });
        assert_eq!(tex.get_width(), 512);
    }

    #[test]
    fn test_device_agent_create_shader() {
        let mut agent = DeviceAgent::default();
        agent.initialize();
        let shader = agent.create_shader(ShaderInfo {
            name: "TestShader".to_string(),
            ..Default::default()
        });
        assert_eq!(shader.get_name(), "TestShader");
    }

    #[test]
    fn test_device_agent_create_command_buffer() {
        let mut agent = DeviceAgent::default();
        agent.initialize();
        let mut cmd = agent.create_command_buffer(CommandBufferInfo::default());
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
    fn test_device_agent_create_command_buffer_raw() {
        let mut agent = DeviceAgent::default();
        agent.initialize();
        let mut cmd = agent.create_command_buffer_raw(CommandBufferInfo::default());
        cmd.begin();
        cmd.draw(&DrawInfo {
            index_count: 3,
            instance_count: 1,
            ..Default::default()
        });
        cmd.end();
        assert_eq!(cmd.num_draw_calls, 1);
    }

    #[test]
    fn test_device_agent_flush_commands() {
        let mut agent = DeviceAgent::default();
        agent.initialize();
        let mut cmd = agent.create_command_buffer(CommandBufferInfo::default());
        cmd.begin();
        cmd.draw(&DrawInfo {
            index_count: 6,
            instance_count: 1,
            ..Default::default()
        });
        cmd.end();
        agent.flush_commands(&mut [cmd]);
        assert_eq!(agent.get_device().num_draw_calls, 1);
        assert_eq!(agent.get_device().num_tris, 2);
    }

    #[test]
    fn test_device_agent_flush_multiple_command_buffers() {
        let mut agent = DeviceAgent::default();
        agent.initialize();
        let mut cmd1 = agent.create_command_buffer(CommandBufferInfo::default());
        let mut cmd2 = agent.create_command_buffer(CommandBufferInfo::default());
        cmd1.begin();
        cmd1.draw(&DrawInfo {
            index_count: 6,
            instance_count: 1,
            ..Default::default()
        });
        cmd1.end();
        cmd2.begin();
        cmd2.draw(&DrawInfo {
            index_count: 12,
            instance_count: 2,
            ..Default::default()
        });
        cmd2.end();
        agent.flush_commands(&mut [cmd1, cmd2]);
        assert_eq!(agent.get_device().num_draw_calls, 2);
        assert_eq!(agent.get_device().num_instances, 3);
    }

    #[test]
    fn test_device_agent_acquire_present() {
        let mut agent = DeviceAgent::default();
        agent.initialize();
        agent.acquire();
        agent.present();
    }

    #[test]
    fn test_device_agent_frame_sync() {
        let mut agent = DeviceAgent::default();
        agent.initialize();
        agent.frame_sync();
    }

    #[test]
    fn test_device_agent_memory_status() {
        let agent = DeviceAgent::default();
        assert_eq!(agent.get_memory_status(), MemoryStatus::Medium);
    }

    #[test]
    fn test_device_agent_create_queue() {
        let mut agent = DeviceAgent::default();
        agent.initialize();
        let queue = agent.create_queue(QueueInfo::default());
        assert!(queue.id > 0);
    }

    #[test]
    fn test_device_agent_create_sampler() {
        let mut agent = DeviceAgent::default();
        agent.initialize();
        let sampler = agent.create_sampler(SamplerInfo::default());
        assert!(sampler.id > 0);
    }

    #[test]
    fn test_device_agent_create_render_pass() {
        let mut agent = DeviceAgent::default();
        agent.initialize();
        let rp = agent.create_render_pass(RenderPassInfo::default());
        assert!(rp.id > 0);
    }

    #[test]
    fn test_device_agent_create_descriptor_set() {
        let mut agent = DeviceAgent::default();
        agent.initialize();
        let layout = agent.create_descriptor_set_layout(DescriptorSetLayoutInfo::default());
        let ds = agent.create_descriptor_set(layout.id);
        assert!(ds.id > 0);
        assert_eq!(ds.layout_id, layout.id);
    }

    #[test]
    fn test_device_agent_destroy() {
        let mut agent = DeviceAgent::default();
        agent.initialize();
        agent.destroy();
        assert!(agent.get_queue().is_none());
        assert!(agent.get_command_buffer().is_none());
        assert!(!agent.is_multithreaded());
    }

    #[test]
    fn test_device_agent_message_queue() {
        let mut agent = DeviceAgent::default();
        agent.initialize();
        let mq = agent.get_main_message_queue();
        assert_eq!(mq.get_pending_count(), 0);
    }

    #[test]
    fn test_device_agent_multithreaded_enqueue() {
        let mut agent = DeviceAgent::default();
        agent.initialize();
        agent.set_multithreaded(true);
        let mut cmd = agent.create_command_buffer(CommandBufferInfo::default());
        cmd.begin();
        cmd.draw(&DrawInfo {
            index_count: 6,
            instance_count: 1,
            ..Default::default()
        });
        cmd.end();
        cmd.flush_messages();
        assert_eq!(cmd.get_num_draw_calls(), 1);
        agent.frame_sync();
    }

    #[test]
    fn test_device_agent_create_all_resource_types() {
        let mut agent = DeviceAgent::default();
        agent.initialize();
        let buf = agent.create_buffer(BufferInfo {
            size: 64,
            ..Default::default()
        });
        let tex = agent.create_texture(TextureInfo {
            width: 64,
            height: 64,
            ..Default::default()
        });
        let shader = agent.create_shader(ShaderInfo {
            name: "s".to_string(),
            ..Default::default()
        });
        let rp = agent.create_render_pass(RenderPassInfo::default());
        let fb = agent.create_framebuffer(FramebufferInfo::default(), 64, 64);
        let ia = agent.create_input_assembler(InputAssemblerInfo::default());
        let layout = agent.create_descriptor_set_layout(DescriptorSetLayoutInfo::default());
        let ds = agent.create_descriptor_set(layout.id);
        let pl = agent.create_pipeline_layout(PipelineLayoutInfo::default());
        let pso = agent.create_pipeline_state(PipelineStateInfo::default());
        let sampler = agent.create_sampler(SamplerInfo::default());
        let q = agent.create_query_pool(QueryPoolInfo::default());
        assert!(buf.id > 0);
        assert!(tex.id > 0);
        assert!(shader.id > 0);
        assert!(rp.id > 0);
        assert!(fb.id > 0);
        assert!(ia.id > 0);
        assert!(layout.id > 0);
        assert!(ds.id > 0);
        assert!(pl.id > 0);
        assert!(pso.id > 0);
        assert!(sampler.id > 0);
        assert!(q.id > 0);
    }

    #[test]
    fn test_device_agent_default_cmd_buff() {
        let mut agent = DeviceAgent::default();
        agent.initialize();
        let cmd = agent.get_command_buffer_mut().unwrap();
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
