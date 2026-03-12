use super::asset::AssetBase;
use crate::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeType {
    Byte = 0,
    UByte = 1,
    Short = 2,
    UShort = 3,
    Int = 4,
    UInt = 5,
    Float = 6,
    Float16 = 7,
}

impl Default for AttributeType {
    fn default() -> Self {
        AttributeType::Float
    }
}

#[derive(Debug, Clone)]
pub struct VertexAttribute {
    pub name: String,
    pub attribute_type: AttributeType,
    pub component_count: u32,
    pub normalized: bool,
    pub offset: u32,
}

impl VertexAttribute {
    pub fn new(name: &str, attribute_type: AttributeType, component_count: u32) -> Self {
        VertexAttribute {
            name: name.to_string(),
            attribute_type,
            component_count,
            normalized: false,
            offset: 0,
        }
    }

    pub fn bytes_per_component(&self) -> u32 {
        match self.attribute_type {
            AttributeType::Byte | AttributeType::UByte => 1,
            AttributeType::Short | AttributeType::UShort | AttributeType::Float16 => 2,
            AttributeType::Int | AttributeType::UInt | AttributeType::Float => 4,
        }
    }

    pub fn stride(&self) -> u32 {
        self.component_count * self.bytes_per_component()
    }
}

#[derive(Debug, Clone)]
pub struct SubMeshInfo {
    pub vertex_bundles_indices: Vec<u32>,
    pub primitives: u32,
    pub index_start: u32,
    pub index_count: u32,
    pub index_end: u32,
}

impl Default for SubMeshInfo {
    fn default() -> Self {
        SubMeshInfo {
            vertex_bundles_indices: Vec::new(),
            primitives: 4,
            index_start: 0,
            index_count: 0,
            index_end: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingSphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Default for BoundingSphere {
    fn default() -> Self {
        BoundingSphere {
            center: Vec3::ZERO,
            radius: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MeshBounds {
    pub min: Vec3,
    pub max: Vec3,
}

impl Default for MeshBounds {
    fn default() -> Self {
        MeshBounds {
            min: Vec3::new(-0.5, -0.5, -0.5),
            max: Vec3::new(0.5, 0.5, 0.5),
        }
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub base: AssetBase,
    pub vertices: Vec<f32>,
    pub indices: Vec<u16>,
    pub indices32: Vec<u32>,
    pub attributes: Vec<VertexAttribute>,
    pub sub_meshes: Vec<SubMeshInfo>,
    pub bounding_sphere: BoundingSphere,
    pub bounds: MeshBounds,
    pub vertex_count: u32,
    pub primitive_count: u32,
}

impl Mesh {
    pub fn new() -> Self {
        Mesh {
            base: AssetBase::new(),
            vertices: Vec::new(),
            indices: Vec::new(),
            indices32: Vec::new(),
            attributes: Vec::new(),
            sub_meshes: Vec::new(),
            bounding_sphere: BoundingSphere::default(),
            bounds: MeshBounds::default(),
            vertex_count: 0,
            primitive_count: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    pub fn get_vertex_count(&self) -> u32 {
        self.vertex_count
    }

    pub fn get_index_count(&self) -> usize {
        self.indices.len() + self.indices32.len()
    }

    pub fn get_sub_mesh_count(&self) -> usize {
        self.sub_meshes.len()
    }

    pub fn get_sub_mesh(&self, index: usize) -> Option<&SubMeshInfo> {
        self.sub_meshes.get(index)
    }

    pub fn add_attribute(&mut self, attr: VertexAttribute) {
        let last_offset = self.attributes.last().map(|a| a.offset + a.stride()).unwrap_or(0);
        let mut attr = attr;
        attr.offset = last_offset;
        self.attributes.push(attr);
    }

    pub fn get_vertex_stride(&self) -> u32 {
        self.attributes.iter().map(|a| a.stride()).sum()
    }

    pub fn update_bounds(&mut self) {
        if self.vertices.is_empty() {
            return;
        }
        let byte_stride = self.get_vertex_stride() as usize;
        if byte_stride < 12 {
            return;
        }
        let float_stride = byte_stride / 4;

        let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vec3::new(-f32::MAX, -f32::MAX, -f32::MAX);

        let mut i = 0;
        while i + 2 < self.vertices.len() {
            let x = self.vertices[i];
            let y = self.vertices[i + 1];
            let z = self.vertices[i + 2];
            min.x = min.x.min(x);
            min.y = min.y.min(y);
            min.z = min.z.min(z);
            max.x = max.x.max(x);
            max.y = max.y.max(y);
            max.z = max.z.max(z);
            i += float_stride;
        }

        self.bounds = MeshBounds { min, max };
        let center = Vec3::new(
            (min.x + max.x) * 0.5,
            (min.y + max.y) * 0.5,
            (min.z + max.z) * 0.5,
        );
        let radius = Vec3::new(
            max.x - center.x,
            max.y - center.y,
            max.z - center.z,
        ).length();
        self.bounding_sphere = BoundingSphere { center, radius };
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_new() {
        let mesh = Mesh::new();
        assert!(mesh.is_empty());
        assert_eq!(mesh.get_sub_mesh_count(), 0);
        assert_eq!(mesh.get_vertex_count(), 0);
    }

    #[test]
    fn test_vertex_attribute() {
        let attr = VertexAttribute::new("a_position", AttributeType::Float, 3);
        assert_eq!(attr.stride(), 12);
    }

    #[test]
    fn test_mesh_add_attribute() {
        let mut mesh = Mesh::new();
        mesh.add_attribute(VertexAttribute::new("a_position", AttributeType::Float, 3));
        mesh.add_attribute(VertexAttribute::new("a_normal", AttributeType::Float, 3));
        assert_eq!(mesh.get_vertex_stride(), 24);
        assert_eq!(mesh.attributes[1].offset, 12);
    }

    #[test]
    fn test_mesh_update_bounds() {
        let mut mesh = Mesh::new();
        mesh.add_attribute(VertexAttribute::new("a_position", AttributeType::Float, 3));
        mesh.vertices = vec![
            -1.0, -1.0, -1.0,
             1.0,  1.0,  1.0,
        ];
        mesh.update_bounds();
        assert!((mesh.bounding_sphere.radius - 3.0_f32.sqrt()).abs() < 0.001);
    }
}
