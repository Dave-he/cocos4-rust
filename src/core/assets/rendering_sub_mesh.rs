/****************************************************************************
Rust port of Cocos Creator RenderingSubMesh
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/
// SPDX-License-Identifier: MIT

use crate::renderer::gfx_base::PrimitiveMode;

#[derive(Debug, Clone, Default)]
pub struct IFlatBuffer {
    pub stride: u32,
    pub count: u32,
    pub buffer: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct IGeometricInfo {
    pub positions: Vec<f32>,
    pub indices: Option<Vec<u32>>,
    pub double_sided: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct RenderingSubMesh {
    pub primitive_mode: PrimitiveMode,
    pub flat_buffers: Vec<IFlatBuffer>,
    pub geometric_info: Option<IGeometricInfo>,
    pub sub_mesh_idx: Option<u32>,
}

impl RenderingSubMesh {
    pub fn new(primitive_mode: PrimitiveMode) -> Self {
        RenderingSubMesh {
            primitive_mode,
            flat_buffers: Vec::new(),
            geometric_info: None,
            sub_mesh_idx: None,
        }
    }

    pub fn invalidate_geometric_info(&mut self) {
        self.geometric_info = None;
    }

    pub fn set_draw_info(
        &mut self,
        _first_vertex: u32,
        _vertex_count: u32,
        _first_index: u32,
        _index_count: u32,
    ) {
        if let Some(info) = self.geometric_info.as_mut() {
            if let Some(_indices) = &mut info.indices {}
        }
    }
}

impl Default for RenderingSubMesh {
    fn default() -> Self {
        RenderingSubMesh {
            primitive_mode: PrimitiveMode::TriangleList,
            flat_buffers: Vec::new(),
            geometric_info: None,
            sub_mesh_idx: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rendering_sub_mesh_new() {
        let sub_mesh = RenderingSubMesh::new(PrimitiveMode::TriangleList);
        assert_eq!(sub_mesh.primitive_mode, PrimitiveMode::TriangleList);
        assert!(sub_mesh.flat_buffers.is_empty());
        assert!(sub_mesh.geometric_info.is_none());
    }

    #[test]
    fn test_rendering_sub_mesh_default() {
        let sub_mesh = RenderingSubMesh::default();
        assert_eq!(sub_mesh.primitive_mode, PrimitiveMode::TriangleList);
    }

    #[test]
    fn test_invalidate_geometric_info() {
        let mut sub_mesh = RenderingSubMesh::default();
        sub_mesh.geometric_info = Some(IGeometricInfo {
            positions: vec![1.0, 2.0, 3.0],
            indices: Some(vec![0, 1, 2]),
            double_sided: None,
        });
        assert!(sub_mesh.geometric_info.is_some());
        sub_mesh.invalidate_geometric_info();
        assert!(sub_mesh.geometric_info.is_none());
    }

    #[test]
    fn test_flat_buffer() {
        let fb = IFlatBuffer {
            stride: 4,
            count: 3,
            buffer: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        };
        assert_eq!(fb.stride, 4);
        assert_eq!(fb.count, 3);
        assert_eq!(fb.buffer.len(), 12);
    }
}
