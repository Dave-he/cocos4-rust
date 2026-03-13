/****************************************************************************
Rust port of Cocos Creator Render Draw Info
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::math::{Color, Mat4, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderDrawInfoType {
    Static = 0,
    Dynamic = 1,
    Particle = 2,
    SubNode = 3,
}

#[derive(Debug, Clone)]
pub struct RenderDrawInfo {
    pub stage: u32,
    pub use_model: bool,
    pub draw_info_type: RenderDrawInfoType,
    pub vertex_offset: u32,
    pub index_offset: u32,
    pub vertex_count: u32,
    pub index_count: u32,
    pub stride: u32,
    pub texture_id: u32,
    pub material_hash: u64,
    pub color: Color,
    pub local_matrix: Mat4,
    pub is_dirty: bool,
    pub is_transparent: bool,
    pub blend_mode: u8,
}

impl RenderDrawInfo {
    pub fn new(stage: u32, use_model: bool) -> Self {
        RenderDrawInfo {
            stage,
            use_model,
            draw_info_type: RenderDrawInfoType::Dynamic,
            vertex_offset: 0,
            index_offset: 0,
            vertex_count: 0,
            index_count: 0,
            stride: 20,
            texture_id: 0,
            material_hash: 0,
            color: Color::WHITE,
            local_matrix: Mat4::IDENTITY,
            is_dirty: true,
            is_transparent: false,
            blend_mode: 0,
        }
    }

    pub fn new_quad() -> Self {
        let mut info = Self::new(0, false);
        info.vertex_count = 4;
        info.index_count = 6;
        info
    }

    pub fn set_texture(&mut self, texture_id: u32) {
        if self.texture_id != texture_id {
            self.texture_id = texture_id;
            self.is_dirty = true;
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
        self.is_dirty = true;
    }

    pub fn set_local_matrix(&mut self, mat: Mat4) {
        self.local_matrix = mat;
        self.is_dirty = true;
    }

    pub fn clear_dirty(&mut self) {
        self.is_dirty = false;
    }

    pub fn set_vertices_and_indices(&mut self, vertex_count: u32, index_count: u32) {
        self.vertex_count = vertex_count;
        self.index_count = index_count;
        self.is_dirty = true;
    }

    pub fn can_merge_with(&self, other: &RenderDrawInfo) -> bool {
        !self.use_model
            && !other.use_model
            && self.texture_id == other.texture_id
            && self.material_hash == other.material_hash
            && self.blend_mode == other.blend_mode
            && self.stage == other.stage
    }
}

impl Default for RenderDrawInfo {
    fn default() -> Self {
        Self::new(0, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_info_new() {
        let info = RenderDrawInfo::new(1, true);
        assert_eq!(info.stage, 1);
        assert!(info.use_model);
        assert!(info.is_dirty);
    }

    #[test]
    fn test_draw_info_set_texture() {
        let mut info = RenderDrawInfo::default();
        info.clear_dirty();
        info.set_texture(42);
        assert_eq!(info.texture_id, 42);
        assert!(info.is_dirty);
    }

    #[test]
    fn test_draw_info_same_texture_no_dirty() {
        let mut info = RenderDrawInfo::default();
        info.set_texture(5);
        info.clear_dirty();
        info.set_texture(5);
        assert!(!info.is_dirty);
    }

    #[test]
    fn test_draw_info_can_merge() {
        let mut a = RenderDrawInfo::default();
        a.texture_id = 1;
        a.material_hash = 100;
        let mut b = RenderDrawInfo::default();
        b.texture_id = 1;
        b.material_hash = 100;
        assert!(a.can_merge_with(&b));
        b.texture_id = 2;
        assert!(!a.can_merge_with(&b));
    }

    #[test]
    fn test_draw_info_quad() {
        let q = RenderDrawInfo::new_quad();
        assert_eq!(q.vertex_count, 4);
        assert_eq!(q.index_count, 6);
    }
}
