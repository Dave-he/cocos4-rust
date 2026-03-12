/****************************************************************************
Rust port of Cocos Creator 3D Mesh
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::math::{Vec2, Vec3, Vec4};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VertexFormat {
    Default = 0,
    Position = 1,
    PositionNormal = 2,
    PositionNormalTex = 3,
    PositionNormalTexTangent = 4,
    PositionSkinning = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexFormat {
    U16 = 0,
    U32 = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    Points = 0,
    Lines = 1,
    LineStrip = 2,
    Triangles = 3,
    TriangleStrip = 4,
}

#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tangent: Vec4,
    pub uv: Vec2,
    pub uv1: Vec2,
    pub color: Vec4,
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            position: Vec3::ZERO,
            normal: Vec3::new(0.0, 1.0, 0.0),
            tangent: Vec4::new(1.0, 0.0, 0.0, 1.0),
            uv: Vec2::ZERO,
            uv1: Vec2::ZERO,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SubMesh {
    pub vertex_buffer: Vec<f32>,
    pub index_buffer_u16: Vec<u16>,
    pub index_buffer_u32: Vec<u32>,
    pub index_format: IndexFormat,
    pub primitive_type: PrimitiveType,
    pub vertex_format: VertexFormat,
    pub vertex_count: u32,
    pub index_count: u32,
    pub min_pos: Vec3,
    pub max_pos: Vec3,
}

impl SubMesh {
    pub fn new() -> Self {
        SubMesh {
            vertex_buffer: Vec::new(),
            index_buffer_u16: Vec::new(),
            index_buffer_u32: Vec::new(),
            index_format: IndexFormat::U16,
            primitive_type: PrimitiveType::Triangles,
            vertex_format: VertexFormat::PositionNormalTex,
            vertex_count: 0,
            index_count: 0,
            min_pos: Vec3::new(f32::MAX, f32::MAX, f32::MAX),
            max_pos: Vec3::new(f32::MIN, f32::MIN, f32::MIN),
        }
    }

    pub fn from_vertices_indices(
        vertices: &[Vertex],
        indices: &[u16],
        primitive_type: PrimitiveType,
    ) -> Self {
        let mut buffer = Vec::with_capacity(vertices.len() * 8);
        let mut min_pos = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max_pos = Vec3::new(f32::MIN, f32::MIN, f32::MIN);

        for v in vertices {
            buffer.push(v.position.x);
            buffer.push(v.position.y);
            buffer.push(v.position.z);
            buffer.push(v.normal.x);
            buffer.push(v.normal.y);
            buffer.push(v.normal.z);
            buffer.push(v.uv.x);
            buffer.push(v.uv.y);

            min_pos.x = min_pos.x.min(v.position.x);
            min_pos.y = min_pos.y.min(v.position.y);
            min_pos.z = min_pos.z.min(v.position.z);
            max_pos.x = max_pos.x.max(v.position.x);
            max_pos.y = max_pos.y.max(v.position.y);
            max_pos.z = max_pos.z.max(v.position.z);
        }

        SubMesh {
            vertex_buffer: buffer,
            index_buffer_u16: indices.to_vec(),
            index_buffer_u32: Vec::new(),
            index_format: IndexFormat::U16,
            primitive_type,
            vertex_format: VertexFormat::PositionNormalTex,
            vertex_count: vertices.len() as u32,
            index_count: indices.len() as u32,
            min_pos,
            max_pos,
        }
    }

    pub fn get_bounds_size(&self) -> Vec3 {
        Vec3::new(
            self.max_pos.x - self.min_pos.x,
            self.max_pos.y - self.min_pos.y,
            self.max_pos.z - self.min_pos.z,
        )
    }

    pub fn get_bounds_center(&self) -> Vec3 {
        Vec3::new(
            (self.min_pos.x + self.max_pos.x) * 0.5,
            (self.min_pos.y + self.max_pos.y) * 0.5,
            (self.min_pos.z + self.max_pos.z) * 0.5,
        )
    }
}

impl Default for SubMesh {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Mesh3D {
    pub name: String,
    pub sub_meshes: Vec<SubMesh>,
    pub joint_names: Vec<String>,
    pub joint_indices: Vec<u8>,
    pub bind_poses: Vec<crate::math::Mat4>,
}

impl Mesh3D {
    pub fn new(name: &str) -> Self {
        Mesh3D {
            name: name.to_string(),
            sub_meshes: Vec::new(),
            joint_names: Vec::new(),
            joint_indices: Vec::new(),
            bind_poses: Vec::new(),
        }
    }

    pub fn add_sub_mesh(&mut self, sub_mesh: SubMesh) {
        self.sub_meshes.push(sub_mesh);
    }

    pub fn get_sub_mesh_count(&self) -> usize {
        self.sub_meshes.len()
    }

    pub fn get_sub_mesh(&self, index: usize) -> Option<&SubMesh> {
        self.sub_meshes.get(index)
    }

    pub fn has_skinning(&self) -> bool {
        !self.joint_names.is_empty()
    }
}

impl Default for Mesh3D {
    fn default() -> Self {
        Self::new("Mesh")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_mesh_new() {
        let sub = SubMesh::new();
        assert_eq!(sub.vertex_count, 0);
        assert_eq!(sub.index_count, 0);
        assert_eq!(sub.primitive_type, PrimitiveType::Triangles);
    }

    #[test]
    fn test_sub_mesh_from_vertices_indices() {
        let vertices = vec![
            Vertex {
                position: Vec3::new(0.0, 0.0, 0.0),
                ..Default::default()
            },
            Vertex {
                position: Vec3::new(1.0, 0.0, 0.0),
                ..Default::default()
            },
            Vertex {
                position: Vec3::new(0.0, 1.0, 0.0),
                ..Default::default()
            },
        ];
        let indices: &[u16] = &[0, 1, 2];
        let sub = SubMesh::from_vertices_indices(&vertices, indices, PrimitiveType::Triangles);
        assert_eq!(sub.vertex_count, 3);
        assert_eq!(sub.index_count, 3);
        assert_eq!(sub.min_pos, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(sub.max_pos, Vec3::new(1.0, 1.0, 0.0));
    }

    #[test]
    fn test_mesh_3d_new() {
        let mut mesh = Mesh3D::new("TestMesh");
        assert_eq!(mesh.name, "TestMesh");
        assert_eq!(mesh.get_sub_mesh_count(), 0);
        assert!(!mesh.has_skinning());

        mesh.add_sub_mesh(SubMesh::default());
        assert_eq!(mesh.get_sub_mesh_count(), 1);
    }
}
