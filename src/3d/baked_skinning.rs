use crate::base::RefCounted;

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

impl std::fmt::Debug for Mesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mesh")
            .field("vertex_format", &self.vertex_format)
            .field("vertex_buffer_id", &self.vertex_buffer_id)
            .field("index_buffer_id", &self.index_buffer_id)
            .finish()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_format() {
        assert_ne!(VertexFormat::Default as u32, VertexFormat::Vec3 as u32);
    }
}
