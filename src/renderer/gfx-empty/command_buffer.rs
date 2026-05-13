/****************************************************************************
Rust port of Cocos Creator GFX Empty Command Buffer
Null/no-op backend for testing and CI.
****************************************************************************/

use crate::renderer::gfx_base::{
    BufferTextureCopy, Color, CommandBufferInfo, CommandBufferState, CommandBufferType,
    DispatchInfo, DrawInfo, DynamicStateFlags, Filter, GfxCommandBuffer, MarkerInfo, Rect,
    StencilFace, TextureBlit, TextureCopy, Viewport,
};

#[derive(Debug)]
pub struct EmptyCommandBuffer {
    inner: GfxCommandBuffer,
}

impl EmptyCommandBuffer {
    pub fn new(info: CommandBufferInfo) -> Self {
        let inner = GfxCommandBuffer::new(0, info);
        Self { inner }
    }

    pub fn initialize(&mut self, info: CommandBufferInfo) {
        self.inner = GfxCommandBuffer::new(0, info);
    }

    pub fn destroy(&mut self) {}

    pub fn begin(&mut self) {
        self.inner.begin();
    }

    pub fn begin_with_render_pass(
        &mut self,
        render_pass: Option<u32>,
        subpass: u32,
        framebuffer: Option<u32>,
    ) {
        self.inner.begin_with_render_pass(render_pass, subpass, framebuffer);
    }

    pub fn end(&mut self) {
        self.inner.end();
    }

    pub fn begin_render_pass(
        &mut self,
        render_pass: u32,
        framebuffer: u32,
        _render_area: &Rect,
        _colors: &[Color],
        _depth: f32,
        _stencil: u32,
    ) {
        self.inner.begin_render_pass(render_pass, framebuffer, &Rect::default(), &[], 1.0, 0);
    }

    pub fn end_render_pass(&mut self) {
        self.inner.end_render_pass();
    }

    pub fn next_subpass(&mut self) {}

    pub fn insert_marker(&mut self, _marker: &MarkerInfo) {}

    pub fn begin_marker(&mut self, _marker: &MarkerInfo) {}

    pub fn end_marker(&mut self) {}

    pub fn bind_pipeline_state(&mut self, pipeline_id: u32) {
        self.inner.bind_pipeline_state(pipeline_id);
    }

    pub fn bind_descriptor_set(&mut self, _set: u32, _descriptor_set_id: u32, _dynamic_offsets: &[u32]) {}

    pub fn bind_input_assembler(&mut self, _ia_id: u32) {}

    pub fn set_viewport(&mut self, _viewport: &Viewport) {}

    pub fn set_scissor(&mut self, _rect: &Rect) {}

    pub fn set_line_width(&mut self, _width: f32) {}

    pub fn set_depth_bias(&mut self, _constant: f32, _clamp: f32, _slope: f32) {}

    pub fn set_blend_constants(&mut self, _constants: &Color) {}

    pub fn set_depth_bound(&mut self, _min_bounds: f32, _max_bounds: f32) {}

    pub fn set_stencil_write_mask(&mut self, _face: StencilFace, _mask: u32) {}

    pub fn set_stencil_compare_mask(&mut self, _face: StencilFace, _ref_val: u32, _mask: u32) {}

    pub fn draw(&mut self, info: &DrawInfo) {
        self.inner.draw(info);
    }

    pub fn update_buffer(&mut self, _buffer_id: u32, _data: &[u8], _size: u32) {}

    pub fn copy_buffers_to_texture(
        &mut self,
        _buffers: &[&[u8]],
        _texture_id: u32,
        _regions: &[BufferTextureCopy],
    ) {}

    pub fn blit_texture(
        &mut self,
        _src_texture: u32,
        _dst_texture: u32,
        _regions: &[TextureBlit],
        _filter: Filter,
    ) {}

    pub fn copy_texture(
        &mut self,
        _src_texture: u32,
        _dst_texture: u32,
        _regions: &[TextureCopy],
    ) {}

    pub fn resolve_texture(
        &mut self,
        _src_texture: u32,
        _dst_texture: u32,
        _regions: &[TextureCopy],
    ) {}

    pub fn copy_buffer_to_buffer(
        &mut self,
        _src_buffer: u32,
        _dst_buffer: u32,
        _regions: &[BufferTextureCopy],
    ) {}

    pub fn copy_buffer_to_texture(
        &mut self,
        _src_buffer: u32,
        _dst_texture: u32,
        _regions: &[BufferTextureCopy],
    ) {}

    pub fn copy_texture_to_buffer(
        &mut self,
        _src_texture: u32,
        _dst_buffer: u32,
        _regions: &[BufferTextureCopy],
    ) {}

    pub fn copy_texture_to_texture(
        &mut self,
        _src_texture: u32,
        _dst_texture: u32,
        _regions: &[TextureCopy],
    ) {}

    pub fn execute(&mut self, _cmd_buffers: &[u32]) {}

    pub fn dispatch(&mut self, _info: &DispatchInfo) {}

    pub fn begin_query(&mut self, _query_pool_id: u32, _id: u32) {}

    pub fn end_query(&mut self, _query_pool_id: u32, _id: u32) {}

    pub fn reset_query_pool(&mut self, _query_pool_id: u32) {}

    pub fn pipeline_barrier(
        &mut self,
        _general_barrier: Option<u32>,
        _buffer_barriers: &[u32],
        _buffers: &[u32],
        _texture_barriers: &[u32],
        _textures: &[u32],
    ) {}

    pub fn set_dynamic_states(&mut self, _flags: DynamicStateFlags) {}

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

    pub fn get_state(&self) -> CommandBufferState {
        self.inner.state.clone()
    }

    pub fn get_id(&self) -> u32 {
        self.inner.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::gfx_base::*;

    #[test]
    fn test_empty_cmd_buffer_new() {
        let cmd = EmptyCommandBuffer::new(CommandBufferInfo::default());
        assert_eq!(cmd.get_type(), CommandBufferType::Primary);
    }

    #[test]
    fn test_empty_cmd_buffer_begin_end() {
        let mut cmd = EmptyCommandBuffer::new(CommandBufferInfo::default());
        cmd.begin();
        assert!(cmd.is_recording());
        cmd.end();
        assert!(!cmd.is_recording());
    }

    #[test]
    fn test_empty_cmd_buffer_draw() {
        let mut cmd = EmptyCommandBuffer::new(CommandBufferInfo::default());
        cmd.begin();
        cmd.draw(&DrawInfo { index_count: 6, instance_count: 1, ..Default::default() });
        assert_eq!(cmd.get_num_draw_calls(), 1);
        assert_eq!(cmd.get_num_tris(), 2);
        cmd.end();
    }

    #[test]
    fn test_empty_cmd_buffer_bind_pipeline() {
        let mut cmd = EmptyCommandBuffer::new(CommandBufferInfo::default());
        cmd.begin();
        cmd.bind_pipeline_state(42);
        cmd.end();
    }

    #[test]
    fn test_empty_cmd_buffer_render_pass() {
        let mut cmd = EmptyCommandBuffer::new(CommandBufferInfo::default());
        cmd.begin();
        cmd.begin_render_pass(1, 1, &Rect::default(), &[], 1.0, 0);
        cmd.end_render_pass();
        cmd.end();
    }

    #[test]
    fn test_empty_cmd_buffer_noop_methods() {
        let mut cmd = EmptyCommandBuffer::new(CommandBufferInfo::default());
        cmd.begin();
        cmd.set_viewport(&Viewport::new(0, 0, 800, 600));
        cmd.set_scissor(&Rect::default());
        cmd.set_line_width(1.0);
        cmd.set_depth_bias(0.0, 0.0, 0.0);
        cmd.set_blend_constants(&Color::default());
        cmd.bind_descriptor_set(0, 1, &[]);
        cmd.bind_input_assembler(1);
        cmd.insert_marker(&MarkerInfo::default());
        cmd.begin_marker(&MarkerInfo::default());
        cmd.end_marker();
        cmd.set_dynamic_states(DynamicStateFlags::NONE);
        cmd.end();
    }
}
