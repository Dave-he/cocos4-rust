/****************************************************************************
Rust port of Cocos Creator GFX Device Validator
Wraps GFX device operations with parameter and lifecycle validation.
****************************************************************************/

use crate::renderer::gfx_base::{
    API, BufferInfo, BufferViewInfo, CommandBufferInfo, DescriptorSetLayoutInfo,
    DeviceInfo, Format, FormatFeature, FramebufferInfo, GfxBuffer, GfxCommandBuffer,
    GfxDescriptorSet, GfxDescriptorSetLayout, GfxDevice, GfxFramebuffer, GfxInputAssembler,
    GfxPipelineLayout, GfxPipelineState, GfxQueryPool, GfxQueue, GfxRenderPass, GfxSampler,
    GfxShader, GfxSwapchain, GfxTexture, InputAssemblerInfo, MemoryStatus,
    PipelineLayoutInfo, PipelineStateInfo, QueryPoolInfo, QueueInfo, RenderPassInfo,
    SamplerInfo, ShaderInfo, SwapchainInfo, TextureInfo, TextureViewInfo,
};

use super::command_buffer::CommandBufferValidator;
use super::resource_tracker::{ResourceTracker, ResourceType};
use super::validation_utils::{ValidationLog, ValidationErrorKind};

pub struct DeviceValidator {
    device: GfxDevice,
    pub resource_tracker: ResourceTracker,
    pub log: ValidationLog,
    pub initialized: bool,
}

impl DeviceValidator {
    pub fn new(info: DeviceInfo) -> Self {
        let device = GfxDevice::new(info);
        DeviceValidator {
            device,
            resource_tracker: ResourceTracker::new(),
            log: ValidationLog::new(),
            initialized: false,
        }
    }

    pub fn from_device(device: GfxDevice) -> Self {
        DeviceValidator {
            device,
            resource_tracker: ResourceTracker::new(),
            log: ValidationLog::new(),
            initialized: false,
        }
    }

    pub fn default() -> Self {
        Self::new(DeviceInfo::default())
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.log.set_enabled(enabled);
        self.resource_tracker.enabled = enabled;
    }

    pub fn initialize(&mut self) -> bool {
        self.validate_binding_mapping_info();
        self.device.api = API::Unknown;
        let all_features = FormatFeature::RENDER_TARGET
            | FormatFeature::SAMPLED_TEXTURE
            | FormatFeature::LINEAR_FILTER
            | FormatFeature::STORAGE_TEXTURE
            | FormatFeature::VERTEX_ATTRIBUTE;
        self.device.set_all_format_features(all_features, true);
        self.initialized = true;
        true
    }

    fn validate_binding_mapping_info(&mut self) {
        let info = &self.device.info;
        if info.binding_mapping_info.set_indices.is_empty() {
            return;
        }
        let flexible_set = *info.binding_mapping_info.set_indices.last().unwrap() as usize;
        if flexible_set < info.binding_mapping_info.max_block_counts.len() {
            if info.binding_mapping_info.max_block_counts[flexible_set] != 0 {
                self.log.error(
                    ValidationErrorKind::Descriptor,
                    "Flexible set maxBlockCounts should be zero",
                );
            }
            if info.binding_mapping_info.max_sampler_texture_counts[flexible_set] != 0 {
                self.log.error(
                    ValidationErrorKind::Descriptor,
                    "Flexible set maxSamplerTextureCounts should be zero",
                );
            }
        }
    }

    pub fn create_command_buffer(&mut self, info: CommandBufferInfo) -> CommandBufferValidator {
        let cmd = self.device.create_command_buffer(info);
        self.resource_tracker.push(ResourceType::CommandBuffer, cmd.id);
        CommandBufferValidator::from_command_buffer(cmd)
    }

    pub fn create_command_buffer_raw(&mut self, info: CommandBufferInfo) -> GfxCommandBuffer {
        let cmd = self.device.create_command_buffer(info);
        self.resource_tracker.push(ResourceType::CommandBuffer, cmd.id);
        cmd
    }

    pub fn create_queue(&mut self, info: QueueInfo) -> GfxQueue {
        let q = self.device.create_queue(info);
        self.resource_tracker.push(ResourceType::Queue, q.id);
        q
    }

    pub fn create_buffer(&mut self, info: BufferInfo) -> GfxBuffer {
        self.validate_buffer_info(&info);
        let buf = self.device.create_buffer(info);
        self.resource_tracker.push(ResourceType::Buffer, buf.id);
        buf
    }

    fn validate_buffer_info(&mut self, info: &BufferInfo) {
        if info.size == 0 {
            self.log.error(ValidationErrorKind::Buffer, "Buffer size must not be zero");
        }
        if info.stride > 0 && info.size % info.stride != 0 {
            self.log.warn(&format!(
                "Buffer size ({}) should be a multiple of stride ({})",
                info.size, info.stride
            ));
        }
    }

    pub fn create_buffer_view(&mut self, info: BufferViewInfo) -> GfxBuffer {
        let buf = self.device.create_buffer_view(info);
        self.resource_tracker.push(ResourceType::Buffer, buf.id);
        buf
    }

    pub fn create_texture(&mut self, info: TextureInfo) -> GfxTexture {
        self.validate_texture_info(&info);
        let tex = self.device.create_texture(info);
        self.resource_tracker.push(ResourceType::Texture, tex.id);
        tex
    }

    fn validate_texture_info(&mut self, info: &TextureInfo) {
        if info.width == 0 || info.height == 0 {
            self.log.error(ValidationErrorKind::Texture, "Texture width and height must not be zero");
        }
        let fmt = info.format;
        let fmt_features = self.device.get_format_features(fmt);
        if !fmt_features.contains(FormatFeature::NONE) && fmt != Format::Unknown {
            if info.usage.contains(crate::renderer::gfx_base::TextureUsage::COLOR_ATTACHMENT)
                && !fmt_features.contains(FormatFeature::RENDER_TARGET)
            {
                self.log.error(
                    ValidationErrorKind::Format,
                    &format!("Format {} does not support RENDER_TARGET", fmt as u32),
                );
            }
        }
    }

    pub fn create_texture_view(&mut self, info: TextureViewInfo) -> GfxTexture {
        let tex = self.device.create_texture_view(info);
        self.resource_tracker.push(ResourceType::Texture, tex.id);
        tex
    }

    pub fn create_shader(&mut self, info: ShaderInfo) -> GfxShader {
        let shader = self.device.create_shader(info);
        self.resource_tracker.push(ResourceType::Shader, shader.id);
        shader
    }

    pub fn create_input_assembler(&mut self, info: InputAssemblerInfo) -> GfxInputAssembler {
        let ia = self.device.create_input_assembler(info);
        self.resource_tracker.push(ResourceType::InputAssembler, ia.id);
        ia
    }

    pub fn create_render_pass(&mut self, info: RenderPassInfo) -> GfxRenderPass {
        let rp = self.device.create_render_pass(info);
        self.resource_tracker.push(ResourceType::RenderPass, rp.id);
        rp
    }

    pub fn create_framebuffer(&mut self, info: FramebufferInfo, width: u32, height: u32) -> GfxFramebuffer {
        let fb = self.device.create_framebuffer(info, width, height);
        self.resource_tracker.push(ResourceType::Framebuffer, fb.id);
        fb
    }

    pub fn create_descriptor_set_layout(&mut self, info: DescriptorSetLayoutInfo) -> GfxDescriptorSetLayout {
        let layout = self.device.create_descriptor_set_layout(info);
        self.resource_tracker.push(ResourceType::DescriptorSetLayout, layout.id);
        layout
    }

    pub fn create_descriptor_set(&mut self, layout_id: u32) -> GfxDescriptorSet {
        let ds = self.device.create_descriptor_set(layout_id);
        self.resource_tracker.push(ResourceType::DescriptorSet, ds.id);
        ds
    }

    pub fn create_pipeline_layout(&mut self, info: PipelineLayoutInfo) -> GfxPipelineLayout {
        let layout = self.device.create_pipeline_layout(info);
        self.resource_tracker.push(ResourceType::PipelineLayout, layout.id);
        layout
    }

    pub fn create_pipeline_state(&mut self, info: PipelineStateInfo) -> GfxPipelineState {
        let pso = self.device.create_pipeline_state(info);
        self.resource_tracker.push(ResourceType::PipelineState, pso.id);
        pso
    }

    pub fn create_sampler(&mut self, info: SamplerInfo) -> GfxSampler {
        if info.address_u != info.address_v || info.address_v != info.address_w {
            self.log.warn("Samplers with different wrapping modes may cause reduced performance");
        }
        let sampler = self.device.create_sampler(info);
        self.resource_tracker.push(ResourceType::Sampler, sampler.id);
        sampler
    }

    pub fn create_swapchain(&mut self, info: SwapchainInfo) -> GfxSwapchain {
        let swapchain = self.device.create_swapchain(info);
        self.resource_tracker.push(ResourceType::Swapchain, swapchain.id);
        swapchain
    }

    pub fn create_query_pool(&mut self, info: QueryPoolInfo) -> GfxQueryPool {
        let pool = self.device.create_query_pool(info);
        self.resource_tracker.push(ResourceType::QueryPool, pool.id);
        pool
    }

    pub fn flush_commands(&mut self, cmd_buffs: &[&GfxCommandBuffer]) {
        self.device.flush_commands(cmd_buffs);
    }

    pub fn acquire(&mut self) {}

    pub fn present(&mut self) {
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    pub fn frame_sync(&mut self) {}

    pub fn get_device(&self) -> &GfxDevice {
        &self.device
    }

    pub fn get_device_mut(&mut self) -> &mut GfxDevice {
        &mut self.device
    }

    pub fn get_memory_status(&self) -> MemoryStatus {
        self.device.get_memory_status()
    }

    pub fn get_resource_tracker(&self) -> &ResourceTracker {
        &self.resource_tracker
    }

    pub fn get_log(&self) -> &ValidationLog {
        &self.log
    }

    pub fn destroy(&mut self) {
        if !self.resource_tracker.check_all_empty() {
            let leaked = self.resource_tracker.get_leaked();
            for (rt, count) in &leaked {
                self.log.error(
                    ValidationErrorKind::ResourceLeak,
                    &format!("{} leaked resource(s) of type {}", count, rt),
                );
            }
        }
        self.device.destroy();
        self.initialized = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::gfx_base::*;

    #[test]
    fn test_device_validator_new() {
        let validator = DeviceValidator::default();
        assert_eq!(validator.get_device().api, API::Unknown);
    }

    #[test]
    fn test_device_validator_initialize() {
        let mut validator = DeviceValidator::default();
        assert!(validator.initialize());
        assert!(validator.initialized);
    }

    #[test]
    fn test_device_validator_create_buffer() {
        let mut validator = DeviceValidator::default();
        validator.initialize();
        let buf = validator.create_buffer(BufferInfo {
            size: 256,
            stride: 4,
            ..Default::default()
        });
        assert_eq!(buf.get_size(), 256);
        assert!(!validator.get_resource_tracker().check_empty(ResourceType::Buffer));
    }

    #[test]
    fn test_device_validator_buffer_zero_size_error() {
        let mut validator = DeviceValidator::default();
        validator.initialize();
        let buf = validator.create_buffer(BufferInfo {
            size: 0,
            ..Default::default()
        });
        assert!(validator.get_log().has_errors());
        validator.resource_tracker.erase(ResourceType::Buffer, buf.id);
    }

    #[test]
    fn test_device_validator_buffer_stride_alignment_warning() {
        let mut validator = DeviceValidator::default();
        validator.initialize();
        let buf = validator.create_buffer(BufferInfo {
            size: 10,
            stride: 4,
            ..Default::default()
        });
        assert!(!validator.get_log().has_errors());
        assert_eq!(validator.get_log().get_warnings().len(), 1);
        validator.resource_tracker.erase(ResourceType::Buffer, buf.id);
    }

    #[test]
    fn test_device_validator_create_texture() {
        let mut validator = DeviceValidator::default();
        validator.initialize();
        let tex = validator.create_texture(TextureInfo {
            width: 512,
            height: 512,
            ..Default::default()
        });
        assert_eq!(tex.get_width(), 512);
        assert!(!validator.get_resource_tracker().check_empty(ResourceType::Texture));
        validator.resource_tracker.erase(ResourceType::Texture, tex.id);
    }

    #[test]
    fn test_device_validator_texture_zero_dimension_error() {
        let mut validator = DeviceValidator::default();
        validator.initialize();
        let tex = validator.create_texture(TextureInfo {
            width: 0,
            height: 512,
            ..Default::default()
        });
        assert!(validator.get_log().has_errors());
        validator.resource_tracker.erase(ResourceType::Texture, tex.id);
    }

    #[test]
    fn test_device_validator_create_shader() {
        let mut validator = DeviceValidator::default();
        validator.initialize();
        let shader = validator.create_shader(ShaderInfo {
            name: "TestShader".to_string(),
            ..Default::default()
        });
        assert_eq!(shader.get_name(), "TestShader");
        validator.resource_tracker.erase(ResourceType::Shader, shader.id);
    }

    #[test]
    fn test_device_validator_sampler_wrapping_warning() {
        let mut validator = DeviceValidator::default();
        validator.initialize();
        let sampler = validator.create_sampler(SamplerInfo {
            address_u: crate::renderer::gfx_base::Address::Wrap,
            address_v: crate::renderer::gfx_base::Address::Clamp,
            address_w: crate::renderer::gfx_base::Address::Wrap,
            ..Default::default()
        });
        assert_eq!(validator.get_log().get_warnings().len(), 1);
        validator.resource_tracker.erase(ResourceType::Sampler, sampler.id);
    }

    #[test]
    fn test_device_validator_create_command_buffer() {
        let mut validator = DeviceValidator::default();
        validator.initialize();
        let mut cmd = validator.create_command_buffer(CommandBufferInfo::default());
        cmd.state_tracker.on_begin();
        cmd.begin();
        cmd.end();
        assert!(!validator.get_resource_tracker().check_empty(ResourceType::CommandBuffer));
    }

    #[test]
    fn test_device_validator_resource_leak_on_destroy() {
        let mut validator = DeviceValidator::default();
        validator.initialize();
        let buf = validator.create_buffer(BufferInfo { size: 64, ..Default::default() });
        validator.destroy();
        assert!(validator.get_log().has_errors());
    }

    #[test]
    fn test_device_validator_no_leak_on_destroy() {
        let mut validator = DeviceValidator::default();
        validator.initialize();
        let buf = validator.create_buffer(BufferInfo { size: 64, ..Default::default() });
        validator.resource_tracker.erase(ResourceType::Buffer, buf.id);
        validator.destroy();
        assert!(!validator.get_log().has_errors());
    }

    #[test]
    fn test_device_validator_disabled_no_checks() {
        let mut validator = DeviceValidator::default();
        validator.set_enabled(false);
        validator.initialize();
        let buf = validator.create_buffer(BufferInfo { size: 0, ..Default::default() });
        assert!(!validator.get_log().has_errors());
    }

    #[test]
    fn test_device_validator_from_device() {
        let device = GfxDevice::default();
        let validator = DeviceValidator::from_device(device);
        assert_eq!(validator.get_device().api, API::Unknown);
    }

    #[test]
    fn test_device_validator_create_descriptor_set() {
        let mut validator = DeviceValidator::default();
        validator.initialize();
        let layout = validator.create_descriptor_set_layout(DescriptorSetLayoutInfo::default());
        let ds = validator.create_descriptor_set(layout.id);
        assert!(ds.id > 0);
        validator.resource_tracker.erase(ResourceType::DescriptorSet, ds.id);
        validator.resource_tracker.erase(ResourceType::DescriptorSetLayout, layout.id);
    }

    #[test]
    fn test_device_validator_create_pipeline_state() {
        let mut validator = DeviceValidator::default();
        validator.initialize();
        let pso = validator.create_pipeline_state(PipelineStateInfo::default());
        assert!(pso.id > 0);
        validator.resource_tracker.erase(ResourceType::PipelineState, pso.id);
    }
}
