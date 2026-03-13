/****************************************************************************
Rust port of Cocos Creator UI Mesh Buffer
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::{RefCounted, RefCountedImpl};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeFormat {
    Float32 = 0,
    Float16 = 1,
    Uint8 = 2,
    Uint16 = 3,
}

#[derive(Debug, Clone)]
pub struct VertexAttribute {
    pub name: String,
    pub format: AttributeFormat,
    pub offset: u32,
    pub stride: u32,
}

impl VertexAttribute {
    pub fn new(name: &str, format: AttributeFormat, offset: u32, stride: u32) -> Self {
        VertexAttribute {
            name: name.to_string(),
            format,
            offset,
            stride,
        }
    }
}

pub trait UIMeshBuffer: RefCounted {
    fn get_vertex_offset(&self) -> u32;
    fn get_index_offset(&self) -> u32;
    fn get_vertex_count(&self) -> u32;
    fn get_index_count(&self) -> u32;
    fn reset(&mut self);
    fn destroy(&mut self);
    fn request_mesh_render_data(&mut self, vertex_count: u32, index_count: u32) -> bool;
}

#[derive(Debug)]
pub struct UIMeshBufferImpl {
    pub vertex_data: Vec<f32>,
    pub index_data: Vec<u16>,
    pub vertex_offset: u32,
    pub index_offset: u32,
    pub attributes: Vec<VertexAttribute>,
    pub stride: u32,
    initialized: bool,
    ref_count: RefCountedImpl,
}

impl UIMeshBufferImpl {
    pub fn new(attributes: Vec<VertexAttribute>, stride: u32) -> Self {
        UIMeshBufferImpl {
            vertex_data: Vec::new(),
            index_data: Vec::new(),
            vertex_offset: 0,
            index_offset: 0,
            attributes,
            stride,
            initialized: false,
            ref_count: RefCountedImpl::new(),
        }
    }

    pub fn initialize(&mut self, vertex_capacity: u32, index_capacity: u32) {
        let float_per_vertex = (self.stride / 4) as usize;
        self.vertex_data = vec![0.0f32; vertex_capacity as usize * float_per_vertex];
        self.index_data = vec![0u16; index_capacity as usize];
        self.initialized = true;
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn write_vertex(&mut self, offset_floats: usize, values: &[f32]) {
        let end = offset_floats + values.len();
        if end <= self.vertex_data.len() {
            self.vertex_data[offset_floats..end].copy_from_slice(values);
        }
    }

    pub fn write_index(&mut self, offset: usize, indices: &[u16]) {
        let end = offset + indices.len();
        if end <= self.index_data.len() {
            self.index_data[offset..end].copy_from_slice(indices);
        }
    }
}

impl Default for UIMeshBufferImpl {
    fn default() -> Self {
        Self::new(Vec::new(), 20)
    }
}

impl RefCounted for UIMeshBufferImpl {
    fn add_ref(&self) { self.ref_count.add_ref(); }
    fn release(&self) { self.ref_count.release(); }
    fn get_ref_count(&self) -> u32 { self.ref_count.get_ref_count() }
    fn is_last_reference(&self) -> bool { self.ref_count.is_last_reference() }
}

impl UIMeshBuffer for UIMeshBufferImpl {
    fn get_vertex_offset(&self) -> u32 { self.vertex_offset }
    fn get_index_offset(&self) -> u32 { self.index_offset }
    fn get_vertex_count(&self) -> u32 {
        if self.stride == 0 { return 0; }
        let floats = self.vertex_data.len() as u32;
        floats / (self.stride / 4)
    }
    fn get_index_count(&self) -> u32 { self.index_data.len() as u32 }

    fn reset(&mut self) {
        self.vertex_offset = 0;
        self.index_offset = 0;
    }

    fn destroy(&mut self) {
        self.vertex_data.clear();
        self.index_data.clear();
        self.vertex_offset = 0;
        self.index_offset = 0;
        self.initialized = false;
    }

    fn request_mesh_render_data(&mut self, vertex_count: u32, index_count: u32) -> bool {
        if !self.initialized {
            return false;
        }
        let float_per_vertex = if self.stride > 0 { self.stride / 4 } else { 1 };
        let needed_verts = (self.vertex_offset + vertex_count * float_per_vertex) as usize;
        let needed_idxs = (self.index_offset + index_count) as usize;
        if needed_verts > self.vertex_data.len() || needed_idxs > self.index_data.len() {
            return false;
        }
        self.vertex_offset += vertex_count * float_per_vertex;
        self.index_offset += index_count;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_buffer_new() {
        let buf = UIMeshBufferImpl::default();
        assert!(!buf.is_initialized());
        assert_eq!(buf.get_vertex_offset(), 0);
        assert_eq!(buf.get_index_offset(), 0);
    }

    #[test]
    fn test_mesh_buffer_initialize() {
        let mut buf = UIMeshBufferImpl::new(Vec::new(), 20);
        buf.initialize(100, 300);
        assert!(buf.is_initialized());
        assert_eq!(buf.get_vertex_count(), 100);
        assert_eq!(buf.get_index_count(), 300);
    }

    #[test]
    fn test_mesh_buffer_request() {
        let mut buf = UIMeshBufferImpl::new(Vec::new(), 20);
        buf.initialize(100, 300);
        let ok = buf.request_mesh_render_data(4, 6);
        assert!(ok);
        assert_eq!(buf.get_vertex_offset(), 4 * 5);
        assert_eq!(buf.get_index_offset(), 6);
    }

    #[test]
    fn test_mesh_buffer_reset() {
        let mut buf = UIMeshBufferImpl::new(Vec::new(), 20);
        buf.initialize(100, 300);
        buf.request_mesh_render_data(4, 6);
        buf.reset();
        assert_eq!(buf.get_vertex_offset(), 0);
        assert_eq!(buf.get_index_offset(), 0);
    }

    #[test]
    fn test_mesh_buffer_write() {
        let mut buf = UIMeshBufferImpl::new(Vec::new(), 20);
        buf.initialize(10, 30);
        buf.write_vertex(0, &[1.0, 2.0, 3.0]);
        assert!((buf.vertex_data[0] - 1.0).abs() < 1e-6);
        assert!((buf.vertex_data[1] - 2.0).abs() < 1e-6);
        buf.write_index(0, &[0, 1, 2]);
        assert_eq!(buf.index_data[0], 0);
        assert_eq!(buf.index_data[2], 2);
    }
}
