/****************************************************************************
Rust port of Cocos Creator GFX Command Buffer Validator
Wraps GfxCommandBuffer with state machine validation checks.
****************************************************************************/

use crate::renderer::gfx_base::{
    BufferTextureCopy, Color, CommandBufferInfo, CommandBufferType, DispatchInfo, DrawInfo,
    DynamicStateFlags, Filter, GfxCommandBuffer, MarkerInfo, Rect, StencilFace, TextureBlit,
    TextureCopy, Viewport,
};

use super::validation_utils::{CommandBufferStateTracker, ValidationErrorKind, ValidationLog};

pub struct CommandBufferValidator {
    pub inner: GfxCommandBuffer,
    pub state_tracker: CommandBufferStateTracker,
    pub log: ValidationLog,
}

impl CommandBufferValidator {
    pub fn new(info: CommandBufferInfo) -> Self {
        let is_primary = info.buffer_type == CommandBufferType::Primary;
        let inner = GfxCommandBuffer::new(0, info);
        let state_tracker = CommandBufferStateTracker::new(is_primary);
        CommandBufferValidator {
            inner,
            state_tracker,
            log: ValidationLog::new(),
        }
    }

    pub fn from_command_buffer(cmd: GfxCommandBuffer) -> Self {
        let is_primary = cmd.info.buffer_type == CommandBufferType::Primary;
        let state_tracker = CommandBufferStateTracker::new(is_primary);
        CommandBufferValidator {
            inner: cmd,
            state_tracker,
            log: ValidationLog::new(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.log.set_enabled(enabled);
    }

    pub fn begin(&mut self) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            self.state_tracker.on_begin();
            self.inner.begin();
        }
    }

    pub fn begin_with_render_pass(
        &mut self,
        render_pass: Option<u32>,
        subpass: u32,
        framebuffer: Option<u32>,
    ) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            if self.state_tracker.is_inside_render_pass() {
                self.log.error(
                    ValidationErrorKind::CommandBufferState,
                    "begin() called while inside render pass",
                );
                return;
            }
            if self.state_tracker.is_primary() && render_pass.is_some() {
                self.log.error(
                    ValidationErrorKind::CommandBufferState,
                    "Primary command buffer cannot inherit render passes",
                );
                return;
            }
            self.state_tracker.on_begin();
            self.inner
                .begin_with_render_pass(render_pass, subpass, framebuffer);
        }
    }

    pub fn end(&mut self) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            if self.state_tracker.is_primary() && self.state_tracker.is_inside_render_pass() {
                self.log.error(
                    ValidationErrorKind::CommandBufferState,
                    "end() called while still inside render pass (primary CB)",
                );
                return;
            }
            self.state_tracker.on_end();
            self.inner.end();
        }
    }

    pub fn begin_render_pass(
        &mut self,
        render_pass: u32,
        framebuffer: u32,
        render_area: &Rect,
        colors: &[Color],
        depth: f32,
        stencil: u32,
    ) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            if !self.state_tracker.is_primary() {
                self.log.error(
                    ValidationErrorKind::CommandBufferState,
                    "beginRenderPass must be recorded in primary command buffers",
                );
                return;
            }
            if self.state_tracker.is_inside_render_pass() {
                self.log.error(
                    ValidationErrorKind::CommandBufferState,
                    "beginRenderPass called while already inside render pass",
                );
                return;
            }
            self.state_tracker.on_begin_render_pass();
            self.inner.begin_render_pass(
                render_pass,
                framebuffer,
                render_area,
                colors,
                depth,
                stencil,
            );
        }
    }

    pub fn end_render_pass(&mut self) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            if !self.state_tracker.is_primary() {
                self.log.error(
                    ValidationErrorKind::CommandBufferState,
                    "endRenderPass must be recorded in primary command buffers",
                );
                return;
            }
            if !self.state_tracker.is_inside_render_pass() {
                self.log.error(
                    ValidationErrorKind::CommandBufferState,
                    "endRenderPass called while not inside render pass",
                );
                return;
            }
            self.state_tracker.on_end_render_pass();
            self.inner.end_render_pass();
        }
    }

    pub fn bind_pipeline_state(&mut self, pipeline_id: u32) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            self.state_tracker.on_bind_pipeline(pipeline_id);
            self.inner.bind_pipeline_state(pipeline_id);
        }
    }

    pub fn bind_descriptor_set(
        &mut self,
        set: u32,
        descriptor_set_id: u32,
        dynamic_offsets: &[u32],
    ) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            self.state_tracker
                .on_bind_descriptor_set(set, descriptor_set_id);
            self.inner
                .bind_descriptor_set(set, descriptor_set_id, dynamic_offsets);
        }
    }

    pub fn bind_input_assembler(&mut self, ia_id: u32) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            self.state_tracker.on_bind_input_assembler(ia_id);
            self.inner.bind_input_assembler(ia_id);
        }
    }

    pub fn set_viewport(&mut self, viewport: &Viewport) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            self.inner.set_viewport(viewport);
        }
    }

    pub fn set_scissor(&mut self, rect: &Rect) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            self.inner.set_scissor(rect);
        }
    }

    pub fn set_line_width(&mut self, width: f32) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            self.inner.set_line_width(width);
        }
    }

    pub fn set_depth_bias(&mut self, constant: f32, clamp: f32, slope: f32) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            self.inner.set_depth_bias(constant, clamp, slope);
        }
    }

    pub fn set_blend_constants(&mut self, constants: &Color) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            self.inner.set_blend_constants(constants);
        }
    }

    pub fn set_depth_bound(&mut self, min_bounds: f32, max_bounds: f32) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            self.inner.set_depth_bound(min_bounds, max_bounds);
        }
    }

    pub fn set_stencil_write_mask(&mut self, face: StencilFace, mask: u32) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            self.inner.set_stencil_write_mask(face, mask);
        }
    }

    pub fn set_stencil_compare_mask(&mut self, face: StencilFace, ref_val: u32, mask: u32) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            self.inner.set_stencil_compare_mask(face, ref_val, mask);
        }
    }

    pub fn draw(&mut self, info: &DrawInfo) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            if !self.state_tracker.is_inside_render_pass() {
                self.log.error(
                    ValidationErrorKind::CommandBufferState,
                    "draw() must be recorded inside render passes",
                );
                return;
            }
            self.inner.draw(info);
        }
    }

    pub fn update_buffer(&mut self, buffer_id: u32, data: &[u8], size: u32) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            if !self.state_tracker.is_primary() {
                self.log.error(
                    ValidationErrorKind::CommandBufferState,
                    "updateBuffer must be recorded in primary command buffers",
                );
                return;
            }
            if self.state_tracker.is_inside_render_pass() {
                self.log.error(
                    ValidationErrorKind::CommandBufferState,
                    "updateBuffer must be recorded outside render passes",
                );
                return;
            }
            self.inner.update_buffer(buffer_id, data, size);
        }
    }

    pub fn copy_buffers_to_texture(
        &mut self,
        buffers: &[&[u8]],
        texture_id: u32,
        regions: &[BufferTextureCopy],
    ) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            if !self.state_tracker.is_primary() {
                self.log.error(
                    ValidationErrorKind::CommandBufferState,
                    "copyBuffersToTexture must be recorded in primary command buffers",
                );
                return;
            }
            if self.state_tracker.is_inside_render_pass() {
                self.log.error(
                    ValidationErrorKind::CommandBufferState,
                    "copyBuffersToTexture must be recorded outside render passes",
                );
                return;
            }
            self.inner
                .copy_buffers_to_texture(buffers, texture_id, regions);
        }
    }

    pub fn dispatch(&mut self, info: &DispatchInfo) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            if self.state_tracker.is_inside_render_pass() {
                self.log.error(
                    ValidationErrorKind::CommandBufferState,
                    "dispatch() must be recorded outside render passes",
                );
                return;
            }
            self.inner.dispatch(info);
        }
    }

    pub fn execute(&mut self, cmd_buffers: &[u32]) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            if !self.state_tracker.is_primary() {
                self.log.error(
                    ValidationErrorKind::CommandBufferState,
                    "execute() must be recorded in primary command buffers",
                );
                return;
            }
            self.inner.execute(cmd_buffers);
        }
    }

    pub fn pipeline_barrier(
        &mut self,
        general_barrier: Option<u32>,
        buffer_barriers: &[u32],
        buffers: &[u32],
        texture_barriers: &[u32],
        textures: &[u32],
    ) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            self.inner.pipeline_barrier(
                general_barrier,
                buffer_barriers,
                buffers,
                texture_barriers,
                textures,
            );
        }
    }

    pub fn next_subpass(&mut self) {
        self.inner.next_subpass();
    }

    pub fn insert_marker(&mut self, marker: &MarkerInfo) {
        self.inner.insert_marker(marker);
    }

    pub fn begin_marker(&mut self, marker: &MarkerInfo) {
        self.inner.begin_marker(marker);
    }

    pub fn end_marker(&mut self) {
        self.inner.end_marker();
    }

    pub fn blit_texture(
        &mut self,
        src_texture: u32,
        dst_texture: u32,
        regions: &[TextureBlit],
        filter: Filter,
    ) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            if self.state_tracker.is_inside_render_pass() {
                self.log.error(
                    ValidationErrorKind::CommandBufferState,
                    "blitTexture must be recorded outside render passes",
                );
                return;
            }
            self.inner
                .blit_texture(src_texture, dst_texture, regions, filter);
        }
    }

    pub fn copy_texture(&mut self, src_texture: u32, dst_texture: u32, regions: &[TextureCopy]) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            if self.state_tracker.is_inside_render_pass() {
                self.log.error(
                    ValidationErrorKind::CommandBufferState,
                    "copyTexture must be recorded outside render passes",
                );
                return;
            }
            self.inner.copy_texture(src_texture, dst_texture, regions);
        }
    }

    pub fn resolve_texture(&mut self, src_texture: u32, dst_texture: u32, regions: &[TextureCopy]) {
        if self.log.assert_inited(
            self.state_tracker.is_inited(),
            "CommandBuffer",
            self.inner.id,
        ) {
            if self.state_tracker.is_inside_render_pass() {
                self.log.error(
                    ValidationErrorKind::CommandBufferState,
                    "resolveTexture must be recorded outside render passes",
                );
                return;
            }
            self.inner
                .resolve_texture(src_texture, dst_texture, regions);
        }
    }

    pub fn begin_query(&mut self, query_pool_id: u32, id: u32) {
        self.inner.begin_query(query_pool_id, id);
    }

    pub fn end_query(&mut self, query_pool_id: u32, id: u32) {
        self.inner.end_query(query_pool_id, id);
    }

    pub fn reset_query_pool(&mut self, query_pool_id: u32) {
        self.inner.reset_query_pool(query_pool_id);
    }

    pub fn set_dynamic_states(&mut self, flags: DynamicStateFlags) {
        self.inner.set_dynamic_states(flags);
    }

    pub fn is_recording(&self) -> bool {
        self.inner.is_recording()
    }

    pub fn get_type(&self) -> CommandBufferType {
        self.inner.get_type()
    }

    pub fn get_num_draw_calls(&self) -> u32 {
        self.inner.num_draw_calls
    }

    pub fn get_num_instances(&self) -> u32 {
        self.inner.num_instances
    }

    pub fn get_num_tris(&self) -> u32 {
        self.inner.num_tris
    }

    pub fn get_id(&self) -> u32 {
        self.inner.id
    }

    pub fn get_inner(&self) -> &GfxCommandBuffer {
        &self.inner
    }

    pub fn get_state_tracker(&self) -> &CommandBufferStateTracker {
        &self.state_tracker
    }

    pub fn get_log(&self) -> &ValidationLog {
        &self.log
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::gfx_base::*;

    #[test]
    fn test_cmd_buffer_validator_begin_end() {
        let mut validator = CommandBufferValidator::new(CommandBufferInfo::default());
        validator.state_tracker.on_begin();
        validator.begin();
        assert!(validator.is_recording());
        validator.end();
        assert!(!validator.is_recording());
    }

    #[test]
    fn test_cmd_buffer_validator_render_pass() {
        let mut validator = CommandBufferValidator::new(CommandBufferInfo::default());
        validator.state_tracker.on_begin();
        validator.begin();
        validator.begin_render_pass(1, 1, &Rect::default(), &[], 1.0, 0);
        assert!(validator.get_state_tracker().is_inside_render_pass());
        validator.end_render_pass();
        assert!(!validator.get_state_tracker().is_inside_render_pass());
        validator.end();
    }

    #[test]
    fn test_cmd_buffer_validator_draw_inside_render_pass() {
        let mut validator = CommandBufferValidator::new(CommandBufferInfo::default());
        validator.state_tracker.on_begin();
        validator.begin();
        validator.begin_render_pass(1, 1, &Rect::default(), &[], 1.0, 0);
        validator.draw(&DrawInfo {
            index_count: 6,
            instance_count: 1,
            ..Default::default()
        });
        assert_eq!(validator.get_num_draw_calls(), 1);
        validator.end_render_pass();
        validator.end();
    }

    #[test]
    fn test_cmd_buffer_validator_draw_outside_render_pass_error() {
        let mut validator = CommandBufferValidator::new(CommandBufferInfo::default());
        validator.state_tracker.on_begin();
        validator.begin();
        validator.draw(&DrawInfo {
            index_count: 6,
            instance_count: 1,
            ..Default::default()
        });
        assert!(validator.get_log().has_errors());
    }

    #[test]
    fn test_cmd_buffer_validator_end_inside_render_pass_error() {
        let mut validator = CommandBufferValidator::new(CommandBufferInfo::default());
        validator.state_tracker.on_begin();
        validator.begin();
        validator.begin_render_pass(1, 1, &Rect::default(), &[], 1.0, 0);
        validator.end();
        assert!(validator.get_log().has_errors());
    }

    #[test]
    fn test_cmd_buffer_validator_bind_pipeline() {
        let mut validator = CommandBufferValidator::new(CommandBufferInfo::default());
        validator.state_tracker.on_begin();
        validator.begin();
        validator.bind_pipeline_state(42);
        assert_eq!(validator.get_state_tracker().get_bound_pipeline(), Some(42));
        validator.end();
    }

    #[test]
    fn test_cmd_buffer_validator_update_buffer_outside_render_pass() {
        let mut validator = CommandBufferValidator::new(CommandBufferInfo::default());
        validator.state_tracker.on_begin();
        validator.begin();
        validator.update_buffer(1, &[0u8; 64], 64);
        assert!(!validator.get_log().has_errors());
        validator.end();
    }

    #[test]
    fn test_cmd_buffer_validator_update_buffer_inside_render_pass_error() {
        let mut validator = CommandBufferValidator::new(CommandBufferInfo::default());
        validator.state_tracker.on_begin();
        validator.begin();
        validator.begin_render_pass(1, 1, &Rect::default(), &[], 1.0, 0);
        validator.update_buffer(1, &[0u8; 64], 64);
        assert!(validator.get_log().has_errors());
        validator.end_render_pass();
        validator.end();
    }

    #[test]
    fn test_cmd_buffer_validator_dispatch_outside_render_pass() {
        let mut validator = CommandBufferValidator::new(CommandBufferInfo::default());
        validator.state_tracker.on_begin();
        validator.begin();
        validator.dispatch(&DispatchInfo {
            group_count_x: 1,
            group_count_y: 1,
            group_count_z: 1,
            ..Default::default()
        });
        assert!(!validator.get_log().has_errors());
        validator.end();
    }

    #[test]
    fn test_cmd_buffer_validator_dispatch_inside_render_pass_error() {
        let mut validator = CommandBufferValidator::new(CommandBufferInfo::default());
        validator.state_tracker.on_begin();
        validator.begin();
        validator.begin_render_pass(1, 1, &Rect::default(), &[], 1.0, 0);
        validator.dispatch(&DispatchInfo::default());
        assert!(validator.get_log().has_errors());
        validator.end_render_pass();
        validator.end();
    }

    #[test]
    fn test_cmd_buffer_validator_from_existing() {
        let mut device = crate::renderer::gfx_base::GfxDevice::default();
        let cmd = device.create_command_buffer(CommandBufferInfo::default());
        let mut validator = CommandBufferValidator::from_command_buffer(cmd);
        validator.state_tracker.on_begin();
        validator.begin();
        validator.end();
    }

    #[test]
    fn test_cmd_buffer_validator_disabled_no_checks() {
        let mut validator = CommandBufferValidator::new(CommandBufferInfo::default());
        validator.set_enabled(false);
        validator.state_tracker.on_begin();
        validator.begin();
        validator.draw(&DrawInfo {
            index_count: 6,
            instance_count: 1,
            ..Default::default()
        });
        assert!(!validator.get_log().has_errors());
        validator.end();
    }
}
