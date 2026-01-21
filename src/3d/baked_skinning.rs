use crate::base::RefCounted;
use crate::core::scene_graph::Node;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VertexFormat {
    Default = 0,
    Vec3 = 1,
}

pub struct Mesh {
    pub vertex_format: VertexFormat,
    pub vertex_buffer_id: u32,
    pub index_buffer_id: u32,
}

pub trait BakedSkinningModel: RefCounted {
    fn get_mesh(&self) -> &Mesh;
}

pub trait MorphModel: RefCounted {
    fn get_mesh(&self) -> &Mesh;
}

pub trait SkinningModel: RefCounted {
    fn get_mesh(&self) -> &Mesh;
}
