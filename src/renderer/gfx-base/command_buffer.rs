/****************************************************************************
Rust port of Cocos Creator GFX Command Buffer
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::{
    BufferTextureCopy, Color, CommandBufferType, DispatchInfo, DrawInfo, DynamicStateFlags,
    Filter, MarkerInfo, QueueType, Rect, StencilFace, TextureBlit, TextureCopy, Viewport,
};

#[derive(Debug, Clone)]
pub struct CommandBufferInfo {
    pub queue_type: QueueType,
    pub buffer_type: CommandBufferType,
}

impl Default for CommandBufferInfo {
    fn default() -> Self {
        CommandBufferInfo {
            queue_type: QueueType::Graphics,
            buffer_type: CommandBufferType::Primary,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandBufferState {
    Idle,
    Recording,
    Executable,
    Pending,
}

#[derive(Debug)]
pub struct GfxCommandBuffer {
    pub id: u32,
    pub info: CommandBufferInfo,
    pub state: CommandBufferState,
    pub num_draw_calls: u32,
    pub num_instances: u32,
    pub num_tris: u32,
    bound_pipeline: Option<u32>,
    bound_render_pass: Option<u32>,
    bound_framebuffer: Option<u32>,
}

impl GfxCommandBuffer {
    pub fn new(id: u32, info: CommandBufferInfo) -> Self {
        GfxCommandBuffer {
            id,
            info,
            state: CommandBufferState::Idle,
            num_draw_calls: 0,
            num_instances: 0,
            num_tris: 0,
            bound_pipeline: None,
            bound_render_pass: None,
            bound_framebuffer: None,
        }
    }

    pub fn begin(&mut self) {
        self.begin_with_render_pass(None, 0, None);
    }

    pub fn begin_with_render_pass(
        &mut self,
        _render_pass: Option<u32>,
        _subpass: u32,
        _framebuffer: Option<u32>,
    ) {
        self.state = CommandBufferState::Recording;
        self.num_draw_calls = 0;
        self.num_instances = 0;
        self.num_tris = 0;
        self.bound_pipeline = None;
        self.bound_render_pass = None;
        self.bound_framebuffer = None;
    }

    pub fn end(&mut self) {
        assert_eq!(self.state, CommandBufferState::Recording);
        self.state = CommandBufferState::Executable;
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
        assert_eq!(self.state, CommandBufferState::Recording);
        self.bound_render_pass = Some(render_pass);
        self.bound_framebuffer = Some(framebuffer);
    }

    pub fn end_render_pass(&mut self) {
        assert_eq!(self.state, CommandBufferState::Recording);
        self.bound_render_pass = None;
        self.bound_framebuffer = None;
    }

    pub fn next_subpass(&mut self) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn insert_marker(&mut self, _marker: &MarkerInfo) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn begin_marker(&mut self, _marker: &MarkerInfo) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn end_marker(&mut self) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn bind_pipeline_state(&mut self, pipeline_id: u32) {
        assert_eq!(self.state, CommandBufferState::Recording);
        self.bound_pipeline = Some(pipeline_id);
    }

    pub fn bind_descriptor_set(
        &mut self,
        _set: u32,
        _descriptor_set_id: u32,
        _dynamic_offsets: &[u32],
    ) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn bind_input_assembler(&mut self, _ia_id: u32) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn set_viewport(&mut self, _viewport: &Viewport) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn set_scissor(&mut self, _rect: &Rect) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn set_line_width(&mut self, _width: f32) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn set_depth_bias(&mut self, _constant: f32, _clamp: f32, _slope: f32) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn set_blend_constants(&mut self, _constants: &Color) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn set_depth_bound(&mut self, _min_bounds: f32, _max_bounds: f32) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn set_stencil_write_mask(&mut self, _face: StencilFace, _mask: u32) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn set_stencil_compare_mask(&mut self, _face: StencilFace, _ref_val: u32, _mask: u32) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn draw(&mut self, info: &DrawInfo) {
        assert_eq!(self.state, CommandBufferState::Recording);
        self.num_draw_calls += 1;
        let instances = info.instance_count.max(1);
        self.num_instances += instances;
        if info.index_count > 0 {
            self.num_tris += info.index_count / 3 * instances;
        } else {
            self.num_tris += info.vertex_count / 3 * instances;
        }
    }

    pub fn update_buffer(&mut self, _buffer_id: u32, _data: &[u8], _size: u32) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn copy_buffers_to_texture(
        &mut self,
        _buffers: &[&[u8]],
        _texture_id: u32,
        _regions: &[BufferTextureCopy],
    ) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn blit_texture(
        &mut self,
        _src_texture: u32,
        _dst_texture: u32,
        _regions: &[TextureBlit],
        _filter: Filter,
    ) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn copy_texture(
        &mut self,
        _src_texture: u32,
        _dst_texture: u32,
        _regions: &[TextureCopy],
    ) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn resolve_texture(
        &mut self,
        _src_texture: u32,
        _dst_texture: u32,
        _regions: &[TextureCopy],
    ) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn execute(&mut self, _cmd_buffers: &[u32]) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn dispatch(&mut self, _info: &DispatchInfo) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn begin_query(&mut self, _query_pool_id: u32, _id: u32) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn end_query(&mut self, _query_pool_id: u32, _id: u32) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn reset_query_pool(&mut self, _query_pool_id: u32) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn pipeline_barrier(
        &mut self,
        _general_barrier: Option<u32>,
        _buffer_barriers: &[u32],
        _buffers: &[u32],
        _texture_barriers: &[u32],
        _textures: &[u32],
    ) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn set_dynamic_states(&mut self, _flags: DynamicStateFlags) {
        assert_eq!(self.state, CommandBufferState::Recording);
    }

    pub fn is_recording(&self) -> bool {
        self.state == CommandBufferState::Recording
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

    pub fn get_type(&self) -> CommandBufferType {
        self.info.buffer_type
    }

    pub fn get_bound_pipeline(&self) -> Option<u32> {
        self.bound_pipeline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_buffer_begin_end() {
        let mut cmd = GfxCommandBuffer::new(1, CommandBufferInfo::default());
        assert_eq!(cmd.state, CommandBufferState::Idle);
        cmd.begin();
        assert!(cmd.is_recording());
        cmd.end();
        assert_eq!(cmd.state, CommandBufferState::Executable);
    }

    #[test]
    fn test_command_buffer_draw() {
        let mut cmd = GfxCommandBuffer::new(1, CommandBufferInfo::default());
        cmd.begin();
        cmd.draw(&DrawInfo { index_count: 6, instance_count: 1, ..Default::default() });
        cmd.draw(&DrawInfo { index_count: 12, instance_count: 2, ..Default::default() });
        assert_eq!(cmd.num_draw_calls, 2);
        assert_eq!(cmd.num_instances, 3);
        assert_eq!(cmd.num_tris, 2 + 8);
    }

    #[test]
    fn test_command_buffer_vertex_draw() {
        let mut cmd = GfxCommandBuffer::new(1, CommandBufferInfo::default());
        cmd.begin();
        cmd.draw(&DrawInfo { vertex_count: 3, instance_count: 1, ..Default::default() });
        assert_eq!(cmd.num_tris, 1);
    }

    #[test]
    fn test_command_buffer_render_pass() {
        let mut cmd = GfxCommandBuffer::new(1, CommandBufferInfo::default());
        cmd.begin();
        cmd.begin_render_pass(1, 1, &Rect::default(), &[], 1.0, 0);
        assert_eq!(cmd.bound_render_pass, Some(1));
        cmd.end_render_pass();
        assert_eq!(cmd.bound_render_pass, None);
    }

    #[test]
    fn test_command_buffer_bind_pipeline() {
        let mut cmd = GfxCommandBuffer::new(1, CommandBufferInfo::default());
        cmd.begin();
        cmd.bind_pipeline_state(42);
        assert_eq!(cmd.get_bound_pipeline(), Some(42));
    }

    #[test]
    fn test_command_buffer_type() {
        let cmd = GfxCommandBuffer::new(1, CommandBufferInfo {
            buffer_type: CommandBufferType::Secondary,
            ..Default::default()
        });
        assert_eq!(cmd.get_type(), CommandBufferType::Secondary);
    }
}
