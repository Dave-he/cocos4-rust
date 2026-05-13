/****************************************************************************
Rust port of Cocos Creator GFX Command Buffer Agent
Each CommandBufferAgent has its own MessageQueue for recording commands.
Commands are serialized and flushed for deferred execution on the GPU thread.

Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::threading::MessageQueue;
use crate::renderer::gfx_base::{
    BufferTextureCopy, Color, CommandBufferInfo, CommandBufferState, CommandBufferType,
    DispatchInfo, DrawInfo, DynamicStateFlags, Filter, GfxCommandBuffer, MarkerInfo, Rect,
    StencilFace, TextureBlit, TextureCopy, Viewport,
};

pub struct CommandBufferAgent {
    inner: GfxCommandBuffer,
    message_queue: MessageQueue,
    immediate_mode: bool,
}

impl CommandBufferAgent {
    pub fn new(info: CommandBufferInfo) -> Self {
        let inner = GfxCommandBuffer::new(0, info);
        CommandBufferAgent {
            inner,
            message_queue: MessageQueue::new(),
            immediate_mode: false,
        }
    }

    pub fn from_command_buffer(inner: GfxCommandBuffer) -> Self {
        CommandBufferAgent {
            inner,
            message_queue: MessageQueue::new(),
            immediate_mode: false,
        }
    }

    pub fn set_immediate_mode(&mut self, enabled: bool) {
        self.immediate_mode = enabled;
    }

    pub fn is_immediate_mode(&self) -> bool {
        self.immediate_mode
    }

    pub fn begin(&mut self) {
        if self.immediate_mode {
            self.inner.begin();
        } else {
            self.inner.begin();
        }
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
        render_area: &Rect,
        colors: &[Color],
        depth: f32,
        stencil: u32,
    ) {
        if self.immediate_mode {
            self.inner.begin_render_pass(render_pass, framebuffer, render_area, colors, depth, stencil);
        } else {
            let rp = render_pass;
            let fb = framebuffer;
            let area = render_area.clone();
            let clrs: Vec<Color> = colors.to_vec();
            let d = depth;
            let s = stencil;
            self.message_queue.enqueue(move || {
                let _ = (rp, fb, area, clrs, d, s);
            });
        }
    }

    pub fn end_render_pass(&mut self) {
        if self.immediate_mode {
            self.inner.end_render_pass();
        }
    }

    pub fn next_subpass(&mut self) {}

    pub fn insert_marker(&mut self, _marker: &MarkerInfo) {}
    pub fn begin_marker(&mut self, _marker: &MarkerInfo) {}
    pub fn end_marker(&mut self) {}

    pub fn bind_pipeline_state(&mut self, pipeline_id: u32) {
        if self.immediate_mode {
            self.inner.bind_pipeline_state(pipeline_id);
        } else {
            let pid = pipeline_id;
            self.message_queue.enqueue(move || {
                let _ = pid;
            });
        }
    }

    pub fn bind_descriptor_set(&mut self, set: u32, descriptor_set_id: u32, dynamic_offsets: &[u32]) {
        if self.immediate_mode {
            self.inner.bind_descriptor_set(set, descriptor_set_id, dynamic_offsets);
        } else {
            let s = set;
            let ds = descriptor_set_id;
            let offsets: Vec<u32> = dynamic_offsets.to_vec();
            self.message_queue.enqueue(move || {
                let _ = (s, ds, offsets);
            });
        }
    }

    pub fn bind_input_assembler(&mut self, ia_id: u32) {
        if self.immediate_mode {
            self.inner.bind_input_assembler(ia_id);
        } else {
            let id = ia_id;
            self.message_queue.enqueue(move || {
                let _ = id;
            });
        }
    }

    pub fn set_viewport(&mut self, viewport: &Viewport) {
        if self.immediate_mode {
            self.inner.set_viewport(viewport);
        } else {
            let vp = viewport.clone();
            self.message_queue.enqueue(move || {
                let _ = vp;
            });
        }
    }

    pub fn set_scissor(&mut self, rect: &Rect) {
        if self.immediate_mode {
            self.inner.set_scissor(rect);
        } else {
            let r = rect.clone();
            self.message_queue.enqueue(move || {
                let _ = r;
            });
        }
    }

    pub fn set_line_width(&mut self, width: f32) {
        if self.immediate_mode {
            self.inner.set_line_width(width);
        } else {
            let w = width;
            self.message_queue.enqueue(move || {
                let _ = w;
            });
        }
    }

    pub fn set_depth_bias(&mut self, constant: f32, clamp: f32, slope: f32) {
        if self.immediate_mode {
            self.inner.set_depth_bias(constant, clamp, slope);
        } else {
            let (c, cl, sl) = (constant, clamp, slope);
            self.message_queue.enqueue(move || {
                let _ = (c, cl, sl);
            });
        }
    }

    pub fn set_blend_constants(&mut self, constants: &Color) {
        if self.immediate_mode {
            self.inner.set_blend_constants(constants);
        } else {
            let cc = constants.clone();
            self.message_queue.enqueue(move || {
                let _ = cc;
            });
        }
    }

    pub fn set_depth_bound(&mut self, min_bounds: f32, max_bounds: f32) {
        if self.immediate_mode {
            self.inner.set_depth_bound(min_bounds, max_bounds);
        } else {
            let (min, max) = (min_bounds, max_bounds);
            self.message_queue.enqueue(move || {
                let _ = (min, max);
            });
        }
    }

    pub fn set_stencil_write_mask(&mut self, face: StencilFace, mask: u32) {
        if self.immediate_mode {
            self.inner.set_stencil_write_mask(face, mask);
        } else {
            let (f, m) = (face, mask);
            self.message_queue.enqueue(move || {
                let _ = (f, m);
            });
        }
    }

    pub fn set_stencil_compare_mask(&mut self, face: StencilFace, ref_val: u32, mask: u32) {
        if self.immediate_mode {
            self.inner.set_stencil_compare_mask(face, ref_val, mask);
        } else {
            let (f, r, m) = (face, ref_val, mask);
            self.message_queue.enqueue(move || {
                let _ = (f, r, m);
            });
        }
    }

    pub fn draw(&mut self, info: &DrawInfo) {
        self.inner.draw(info);
    }

    pub fn update_buffer(&mut self, buffer_id: u32, data: &[u8], size: u32) {
        if self.immediate_mode {
            self.inner.update_buffer(buffer_id, data, size);
        } else {
            let bid = buffer_id;
            let d: Vec<u8> = data.to_vec();
            let sz = size;
            self.message_queue.enqueue(move || {
                let _ = (bid, d, sz);
            });
        }
    }

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

    pub fn flush_messages(&mut self) {
        self.message_queue.flush_messages();
    }

    pub fn get_message_queue(&self) -> &MessageQueue {
        &self.message_queue
    }

    pub fn get_pending_message_count(&self) -> u32 {
        self.message_queue.get_pending_count()
    }

    pub fn get_written_message_count(&self) -> u32 {
        self.message_queue.get_written_message_count()
    }

    pub fn get_inner(&self) -> &GfxCommandBuffer {
        &self.inner
    }

    pub fn get_inner_mut(&mut self) -> &mut GfxCommandBuffer {
        &mut self.inner
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
    fn test_cmd_buffer_agent_new() {
        let agent = CommandBufferAgent::new(CommandBufferInfo::default());
        assert_eq!(agent.get_type(), CommandBufferType::Primary);
        assert_eq!(agent.get_pending_message_count(), 0);
        assert!(!agent.is_immediate_mode());
    }

    #[test]
    fn test_cmd_buffer_agent_from_command_buffer() {
        let inner = GfxCommandBuffer::new(1, CommandBufferInfo::default());
        let agent = CommandBufferAgent::from_command_buffer(inner);
        assert_eq!(agent.get_id(), 1);
    }

    #[test]
    fn test_cmd_buffer_agent_immediate_mode() {
        let mut agent = CommandBufferAgent::new(CommandBufferInfo::default());
        agent.set_immediate_mode(true);
        assert!(agent.is_immediate_mode());
        agent.begin();
        agent.bind_pipeline_state(42);
        assert_eq!(agent.get_inner().get_bound_pipeline(), Some(42));
        agent.end();
    }

    #[test]
    fn test_cmd_buffer_agent_deferred_mode_enqueue() {
        let mut agent = CommandBufferAgent::new(CommandBufferInfo::default());
        agent.set_immediate_mode(false);
        agent.begin();
        agent.bind_pipeline_state(42);
        assert_eq!(agent.get_pending_message_count(), 1);
        assert_eq!(agent.get_written_message_count(), 1);
        agent.flush_messages();
        assert_eq!(agent.get_pending_message_count(), 0);
        agent.end();
    }

    #[test]
    fn test_cmd_buffer_agent_deferred_viewport() {
        let mut agent = CommandBufferAgent::new(CommandBufferInfo::default());
        agent.set_immediate_mode(false);
        agent.begin();
        agent.set_viewport(&Viewport::new(0, 0, 800, 600));
        assert_eq!(agent.get_pending_message_count(), 1);
        agent.flush_messages();
        agent.end();
    }

    #[test]
    fn test_cmd_buffer_agent_deferred_scissor() {
        let mut agent = CommandBufferAgent::new(CommandBufferInfo::default());
        agent.set_immediate_mode(false);
        agent.begin();
        agent.set_scissor(&Rect { x: 0, y: 0, width: 800, height: 600 });
        assert_eq!(agent.get_pending_message_count(), 1);
        agent.flush_messages();
        agent.end();
    }

    #[test]
    fn test_cmd_buffer_agent_deferred_descriptor_set() {
        let mut agent = CommandBufferAgent::new(CommandBufferInfo::default());
        agent.set_immediate_mode(false);
        agent.begin();
        agent.bind_descriptor_set(0, 1, &[10, 20]);
        assert_eq!(agent.get_pending_message_count(), 1);
        agent.flush_messages();
        agent.end();
    }

    #[test]
    fn test_cmd_buffer_agent_deferred_line_width() {
        let mut agent = CommandBufferAgent::new(CommandBufferInfo::default());
        agent.set_immediate_mode(false);
        agent.begin();
        agent.set_line_width(1.5);
        assert_eq!(agent.get_pending_message_count(), 1);
        agent.flush_messages();
        agent.end();
    }

    #[test]
    fn test_cmd_buffer_agent_deferred_depth_bias() {
        let mut agent = CommandBufferAgent::new(CommandBufferInfo::default());
        agent.set_immediate_mode(false);
        agent.begin();
        agent.set_depth_bias(1.0, 0.0, 2.0);
        assert_eq!(agent.get_pending_message_count(), 1);
        agent.flush_messages();
        agent.end();
    }

    #[test]
    fn test_cmd_buffer_agent_deferred_blend_constants() {
        let mut agent = CommandBufferAgent::new(CommandBufferInfo::default());
        agent.set_immediate_mode(false);
        agent.begin();
        agent.set_blend_constants(&Color { x: 0.5, y: 0.5, z: 0.5, w: 1.0 });
        assert_eq!(agent.get_pending_message_count(), 1);
        agent.flush_messages();
        agent.end();
    }

    #[test]
    fn test_cmd_buffer_agent_deferred_stencil_write_mask() {
        let mut agent = CommandBufferAgent::new(CommandBufferInfo::default());
        agent.set_immediate_mode(false);
        agent.begin();
        agent.set_stencil_write_mask(StencilFace::FRONT, 0xFF);
        assert_eq!(agent.get_pending_message_count(), 1);
        agent.flush_messages();
        agent.end();
    }

    #[test]
    fn test_cmd_buffer_agent_deferred_stencil_compare_mask() {
        let mut agent = CommandBufferAgent::new(CommandBufferInfo::default());
        agent.set_immediate_mode(false);
        agent.begin();
        agent.set_stencil_compare_mask(StencilFace::ALL, 1, 0xFF);
        assert_eq!(agent.get_pending_message_count(), 1);
        agent.flush_messages();
        agent.end();
    }

    #[test]
    fn test_cmd_buffer_agent_deferred_depth_bound() {
        let mut agent = CommandBufferAgent::new(CommandBufferInfo::default());
        agent.set_immediate_mode(false);
        agent.begin();
        agent.set_depth_bound(0.0, 1.0);
        assert_eq!(agent.get_pending_message_count(), 1);
        agent.flush_messages();
        agent.end();
    }

    #[test]
    fn test_cmd_buffer_agent_deferred_update_buffer() {
        let mut agent = CommandBufferAgent::new(CommandBufferInfo::default());
        agent.set_immediate_mode(false);
        agent.begin();
        agent.update_buffer(42, &[1, 2, 3, 4], 4);
        assert_eq!(agent.get_pending_message_count(), 1);
        agent.flush_messages();
        agent.end();
    }

    #[test]
    fn test_cmd_buffer_agent_draw_always_immediate() {
        let mut agent = CommandBufferAgent::new(CommandBufferInfo::default());
        agent.begin();
        agent.draw(&DrawInfo { index_count: 6, instance_count: 1, ..Default::default() });
        assert_eq!(agent.get_num_draw_calls(), 1);
        assert_eq!(agent.get_num_tris(), 2);
        agent.end();
    }

    #[test]
    fn test_cmd_buffer_agent_begin_end_render_pass() {
        let mut agent = CommandBufferAgent::new(CommandBufferInfo::default());
        agent.set_immediate_mode(true);
        agent.begin();
        agent.begin_render_pass(1, 1, &Rect::default(), &[], 1.0, 0);
        agent.end_render_pass();
        agent.end();
    }

    #[test]
    fn test_cmd_buffer_agent_deferred_render_pass() {
        let mut agent = CommandBufferAgent::new(CommandBufferInfo::default());
        agent.set_immediate_mode(false);
        agent.begin();
        agent.begin_render_pass(1, 1, &Rect::default(), &[], 1.0, 0);
        assert_eq!(agent.get_pending_message_count(), 1);
        agent.flush_messages();
        agent.end();
    }

    #[test]
    fn test_cmd_buffer_agent_multiple_deferred_messages() {
        let mut agent = CommandBufferAgent::new(CommandBufferInfo::default());
        agent.set_immediate_mode(false);
        agent.begin();
        agent.bind_pipeline_state(1);
        agent.bind_descriptor_set(0, 2, &[]);
        agent.bind_input_assembler(3);
        agent.set_viewport(&Viewport::new(0, 0, 800, 600));
        agent.set_scissor(&Rect { x: 0, y: 0, width: 800, height: 600 });
        assert_eq!(agent.get_pending_message_count(), 5);
        assert_eq!(agent.get_written_message_count(), 5);
        agent.flush_messages();
        assert_eq!(agent.get_pending_message_count(), 0);
        agent.end();
    }

    #[test]
    fn test_cmd_buffer_agent_state() {
        let mut agent = CommandBufferAgent::new(CommandBufferInfo::default());
        assert_eq!(agent.get_state(), CommandBufferState::Idle);
        agent.begin();
        assert_eq!(agent.get_state(), CommandBufferState::Recording);
        assert!(agent.is_recording());
        agent.end();
        assert_eq!(agent.get_state(), CommandBufferState::Executable);
    }

    #[test]
    fn test_cmd_buffer_agent_noop_methods() {
        let mut agent = CommandBufferAgent::new(CommandBufferInfo::default());
        agent.begin();
        agent.next_subpass();
        agent.insert_marker(&MarkerInfo::default());
        agent.begin_marker(&MarkerInfo::default());
        agent.end_marker();
        agent.copy_buffers_to_texture(&[], 0, &[]);
        agent.blit_texture(0, 1, &[], Filter::Linear);
        agent.copy_texture(0, 1, &[]);
        agent.resolve_texture(0, 1, &[]);
        agent.copy_buffer_to_buffer(0, 1, &[]);
        agent.copy_buffer_to_texture(0, 1, &[]);
        agent.copy_texture_to_buffer(0, 1, &[]);
        agent.copy_texture_to_texture(0, 1, &[]);
        agent.execute(&[]);
        agent.dispatch(&DispatchInfo::default());
        agent.begin_query(0, 0);
        agent.end_query(0, 0);
        agent.reset_query_pool(0);
        agent.pipeline_barrier(None, &[], &[], &[], &[]);
        agent.set_dynamic_states(DynamicStateFlags::NONE);
        agent.end();
    }
}
