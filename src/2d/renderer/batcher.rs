/****************************************************************************
Rust port of Cocos Creator 2D Batcher System
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::math::{Color, Mat4, Vec2};
use super::draw_info::RenderDrawInfo;
use super::ui_mesh::{UIMeshBuffer, UIMeshBufferImpl, VertexAttribute, AttributeFormat};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Batcher2DType {
    Dynamic = 0,
    World = 1,
}

#[derive(Debug, Clone)]
struct BatchCommand {
    pub draw_info_index: usize,
    pub vertex_start: u32,
    pub vertex_count: u32,
    pub index_start: u32,
    pub index_count: u32,
    pub texture_id: u32,
    pub material_hash: u64,
}

#[derive(Debug)]
pub struct Batcher2D {
    pub batcher_2d_type: Batcher2DType,
    mesh_buffers: Vec<UIMeshBufferImpl>,
    draw_infos: Vec<RenderDrawInfo>,
    batch_commands: Vec<BatchCommand>,
    vertex_count: u32,
    index_count: u32,
    initialized: bool,
    dirty: bool,
}

impl Batcher2D {
    pub fn new() -> Self {
        Batcher2D {
            batcher_2d_type: Batcher2DType::Dynamic,
            mesh_buffers: Vec::new(),
            draw_infos: Vec::new(),
            batch_commands: Vec::new(),
            vertex_count: 0,
            index_count: 0,
            initialized: false,
            dirty: false,
        }
    }

    pub fn initialize(&mut self, vertex_capacity: u32, index_capacity: u32) {
        let attrs = vec![
            VertexAttribute::new("a_position", AttributeFormat::Float32, 0, 20),
            VertexAttribute::new("a_texCoord", AttributeFormat::Float32, 8, 20),
            VertexAttribute::new("a_color", AttributeFormat::Uint8, 16, 20),
        ];
        let mut buf = UIMeshBufferImpl::new(attrs, 20);
        buf.initialize(vertex_capacity, index_capacity);
        self.mesh_buffers.push(buf);
        self.initialized = true;
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn add_draw_info(&mut self, info: RenderDrawInfo) -> usize {
        let idx = self.draw_infos.len();
        self.vertex_count += info.vertex_count;
        self.index_count += info.index_count;
        self.draw_infos.push(info);
        self.dirty = true;
        idx
    }

    pub fn update_draw_info(&mut self, idx: usize, info: RenderDrawInfo) {
        if idx < self.draw_infos.len() {
            self.draw_infos[idx] = info;
            self.dirty = true;
        }
    }

    pub fn remove_draw_info(&mut self, idx: usize) {
        if idx < self.draw_infos.len() {
            let removed = self.draw_infos.remove(idx);
            self.vertex_count = self.vertex_count.saturating_sub(removed.vertex_count);
            self.index_count = self.index_count.saturating_sub(removed.index_count);
            self.dirty = true;
        }
    }

    pub fn flush(&mut self) {
        if !self.dirty || !self.initialized {
            return;
        }
        self.batch_commands.clear();
        if let Some(buf) = self.mesh_buffers.first_mut() {
            buf.reset();
        }

        let mut current_vertex: u32 = 0;
        let mut current_index: u32 = 0;

        let mut i = 0;
        while i < self.draw_infos.len() {
            let info = &self.draw_infos[i];
            let mut merged_vertex = info.vertex_count;
            let mut merged_index = info.index_count;
            let mut j = i + 1;
            while j < self.draw_infos.len() {
                if info.can_merge_with(&self.draw_infos[j]) {
                    merged_vertex += self.draw_infos[j].vertex_count;
                    merged_index += self.draw_infos[j].index_count;
                    j += 1;
                } else {
                    break;
                }
            }

            self.batch_commands.push(BatchCommand {
                draw_info_index: i,
                vertex_start: current_vertex,
                vertex_count: merged_vertex,
                index_start: current_index,
                index_count: merged_index,
                texture_id: info.texture_id,
                material_hash: info.material_hash,
            });

            current_vertex += merged_vertex;
            current_index += merged_index;
            i = j;
        }

        self.dirty = false;
    }

    pub fn get_draw_call_count(&self) -> usize {
        self.batch_commands.len()
    }

    pub fn get_total_vertex_count(&self) -> u32 {
        self.vertex_count
    }

    pub fn get_total_index_count(&self) -> u32 {
        self.index_count
    }

    pub fn get_draw_info_count(&self) -> usize {
        self.draw_infos.len()
    }

    pub fn clear(&mut self) {
        self.draw_infos.clear();
        self.batch_commands.clear();
        self.vertex_count = 0;
        self.index_count = 0;
        self.dirty = false;
        if let Some(buf) = self.mesh_buffers.first_mut() {
            buf.reset();
        }
    }

    pub fn destroy(&mut self) {
        self.clear();
        for buf in &mut self.mesh_buffers {
            buf.destroy();
        }
        self.mesh_buffers.clear();
        self.initialized = false;
    }

    pub fn update(&mut self) {
        if self.dirty {
            self.fill_buffers_and_merge_batches();
        }
    }

    pub fn upload_buffers(&mut self) {
        for buf in &mut self.mesh_buffers {
            buf.upload();
        }
    }

    pub fn reset(&mut self) {
        self.batch_commands.clear();
        for buf in &mut self.mesh_buffers {
            buf.reset();
        }
        self.dirty = true;
    }

    pub fn fill_buffers_and_merge_batches(&mut self) {
        self.flush();
    }

    pub fn generate_batch_for_draw_info(&mut self, draw_info_idx: usize) {
        if draw_info_idx >= self.draw_infos.len() { return; }
        let info = &self.draw_infos[draw_info_idx];
        let cmd = BatchCommand {
            draw_info_index: draw_info_idx,
            vertex_start: self.vertex_count,
            vertex_count: info.vertex_count,
            index_start: self.index_count,
            index_count: info.index_count,
            texture_id: info.texture_id,
            material_hash: info.material_hash,
        };
        self.batch_commands.push(cmd);
    }

    pub fn reset_render_states(&mut self) {
        self.batch_commands.clear();
        self.dirty = true;
    }

    pub fn get_mesh_buffer_count(&self) -> usize {
        self.mesh_buffers.len()
    }

    pub fn sync_mesh_buffers(&mut self, buffers: Vec<UIMeshBufferImpl>) {
        self.mesh_buffers = buffers;
    }
}

impl Default for Batcher2D {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Color;

    fn make_draw_info(tex: u32, vc: u32, ic: u32) -> RenderDrawInfo {
        let mut info = RenderDrawInfo::default();
        info.texture_id = tex;
        info.vertex_count = vc;
        info.index_count = ic;
        info
    }

    #[test]
    fn test_batcher_new() {
        let b = Batcher2D::new();
        assert!(!b.is_initialized());
        assert_eq!(b.get_draw_info_count(), 0);
    }

    #[test]
    fn test_batcher_initialize() {
        let mut b = Batcher2D::new();
        b.initialize(1000, 3000);
        assert!(b.is_initialized());
    }

    #[test]
    fn test_add_draw_info() {
        let mut b = Batcher2D::new();
        b.initialize(1000, 3000);
        let idx = b.add_draw_info(make_draw_info(1, 4, 6));
        assert_eq!(idx, 0);
        assert_eq!(b.get_draw_info_count(), 1);
        assert_eq!(b.get_total_vertex_count(), 4);
        assert_eq!(b.get_total_index_count(), 6);
    }

    #[test]
    fn test_flush_merges_same_texture() {
        let mut b = Batcher2D::new();
        b.initialize(1000, 3000);
        b.add_draw_info(make_draw_info(1, 4, 6));
        b.add_draw_info(make_draw_info(1, 4, 6));
        b.add_draw_info(make_draw_info(1, 4, 6));
        b.flush();
        assert_eq!(b.get_draw_call_count(), 1);
    }

    #[test]
    fn test_flush_separates_diff_texture() {
        let mut b = Batcher2D::new();
        b.initialize(1000, 3000);
        b.add_draw_info(make_draw_info(1, 4, 6));
        b.add_draw_info(make_draw_info(2, 4, 6));
        b.flush();
        assert_eq!(b.get_draw_call_count(), 2);
    }

    #[test]
    fn test_remove_draw_info() {
        let mut b = Batcher2D::new();
        b.initialize(1000, 3000);
        b.add_draw_info(make_draw_info(1, 4, 6));
        b.add_draw_info(make_draw_info(2, 4, 6));
        b.remove_draw_info(0);
        assert_eq!(b.get_draw_info_count(), 1);
        assert_eq!(b.get_total_vertex_count(), 4);
    }

    #[test]
    fn test_clear() {
        let mut b = Batcher2D::new();
        b.initialize(1000, 3000);
        b.add_draw_info(make_draw_info(1, 4, 6));
        b.flush();
        b.clear();
        assert_eq!(b.get_draw_info_count(), 0);
        assert_eq!(b.get_draw_call_count(), 0);
        assert_eq!(b.get_total_vertex_count(), 0);
    }

    #[test]
    fn test_flush_idempotent() {
        let mut b = Batcher2D::new();
        b.initialize(1000, 3000);
        b.add_draw_info(make_draw_info(1, 4, 6));
        b.flush();
        let count = b.get_draw_call_count();
        b.flush();
        assert_eq!(b.get_draw_call_count(), count);
    }
}
