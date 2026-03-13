/****************************************************************************
Rust port of Cocos Creator 3D Asset System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::{RefCounted, RefCountedImpl};
use crate::math::{Vec2, Vec3, Vec4, Mat4};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
    Unknown = 0,
    Texture = 1,
    Material = 2,
    Mesh = 3,
    Model = 4,
    Animation = 5,
    Effect = 6,
    Image = 7,
    Font = 8,
}

pub trait Asset: RefCounted {
    fn get_asset_type(&self) -> AssetType;
    fn get_name(&self) -> &str;
    fn is_loaded(&self) -> bool;
    fn destroy(&mut self);
}

pub trait Mesh: RefCounted {
    fn get_vertex_count(&self) -> u32;
    fn get_index_count(&self) -> u32;
    fn get_sub_mesh_count(&self) -> usize;
    fn get_name(&self) -> &str;
}

pub trait Model: RefCounted {
    fn get_name(&self) -> &str;
    fn is_enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
}

#[derive(Debug)]
pub struct AssetBase {
    pub name: String,
    pub asset_type: AssetType,
    pub loaded: bool,
    pub uuid: String,
    ref_count: RefCountedImpl,
}

impl AssetBase {
    pub fn new(name: &str, asset_type: AssetType) -> Self {
        AssetBase {
            name: name.to_string(),
            asset_type,
            loaded: false,
            uuid: String::new(),
            ref_count: RefCountedImpl::new(),
        }
    }

    pub fn with_uuid(name: &str, asset_type: AssetType, uuid: &str) -> Self {
        AssetBase {
            name: name.to_string(),
            asset_type,
            loaded: false,
            uuid: uuid.to_string(),
            ref_count: RefCountedImpl::new(),
        }
    }

    pub fn set_loaded(&mut self, loaded: bool) {
        self.loaded = loaded;
    }

    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }
}

impl RefCounted for AssetBase {
    fn add_ref(&self) { self.ref_count.add_ref(); }
    fn release(&self) { self.ref_count.release(); }
    fn get_ref_count(&self) -> u32 { self.ref_count.get_ref_count() }
    fn is_last_reference(&self) -> bool { self.ref_count.is_last_reference() }
}

impl Asset for AssetBase {
    fn get_asset_type(&self) -> AssetType { self.asset_type }
    fn get_name(&self) -> &str { &self.name }
    fn is_loaded(&self) -> bool { self.loaded }
    fn destroy(&mut self) { self.loaded = false; }
}

#[derive(Debug, Clone)]
pub struct SubMeshInfo {
    pub vertex_count: u32,
    pub index_count: u32,
    pub vertex_offset: u32,
    pub index_offset: u32,
    pub material_index: u32,
    pub bounding_radius: f32,
    pub min_pos: Vec3,
    pub max_pos: Vec3,
}

impl Default for SubMeshInfo {
    fn default() -> Self {
        SubMeshInfo {
            vertex_count: 0,
            index_count: 0,
            vertex_offset: 0,
            index_offset: 0,
            material_index: 0,
            bounding_radius: 0.0,
            min_pos: Vec3::new(-0.5, -0.5, -0.5),
            max_pos: Vec3::new(0.5, 0.5, 0.5),
        }
    }
}

#[derive(Debug)]
pub struct MeshAsset {
    base: AssetBase,
    pub sub_meshes: Vec<SubMeshInfo>,
    pub vertex_data: Vec<f32>,
    pub index_data_u16: Vec<u16>,
    pub index_data_u32: Vec<u32>,
    pub has_normals: bool,
    pub has_uvs: bool,
    pub has_tangents: bool,
    pub has_colors: bool,
    pub has_skinning: bool,
}

impl MeshAsset {
    pub fn new(name: &str) -> Self {
        MeshAsset {
            base: AssetBase::new(name, AssetType::Mesh),
            sub_meshes: Vec::new(),
            vertex_data: Vec::new(),
            index_data_u16: Vec::new(),
            index_data_u32: Vec::new(),
            has_normals: false,
            has_uvs: false,
            has_tangents: false,
            has_colors: false,
            has_skinning: false,
        }
    }

    pub fn add_sub_mesh(&mut self, sub_mesh: SubMeshInfo) {
        self.sub_meshes.push(sub_mesh);
    }

    pub fn set_vertex_data(&mut self, data: Vec<f32>) {
        self.vertex_data = data;
        self.base.loaded = true;
    }

    pub fn set_index_data_u16(&mut self, data: Vec<u16>) {
        self.index_data_u16 = data;
    }

    pub fn set_index_data_u32(&mut self, data: Vec<u32>) {
        self.index_data_u32 = data;
    }

    pub fn total_vertex_count(&self) -> u32 {
        self.sub_meshes.iter().map(|s| s.vertex_count).sum()
    }

    pub fn total_index_count(&self) -> u32 {
        self.sub_meshes.iter().map(|s| s.index_count).sum()
    }

    pub fn get_uuid(&self) -> &str {
        self.base.get_uuid()
    }
}

impl RefCounted for MeshAsset {
    fn add_ref(&self) { self.base.ref_count.add_ref(); }
    fn release(&self) { self.base.ref_count.release(); }
    fn get_ref_count(&self) -> u32 { self.base.ref_count.get_ref_count() }
    fn is_last_reference(&self) -> bool { self.base.ref_count.is_last_reference() }
}

impl Mesh for MeshAsset {
    fn get_vertex_count(&self) -> u32 { self.total_vertex_count() }
    fn get_index_count(&self) -> u32 { self.total_index_count() }
    fn get_sub_mesh_count(&self) -> usize { self.sub_meshes.len() }
    fn get_name(&self) -> &str { &self.base.name }
}

#[derive(Debug)]
pub struct ModelAsset {
    base: AssetBase,
    pub mesh_uuids: Vec<String>,
    pub material_uuids: Vec<String>,
    pub skeleton_uuid: Option<String>,
    pub node_count: u32,
    pub skin_names: Vec<String>,
}

impl ModelAsset {
    pub fn new(name: &str) -> Self {
        ModelAsset {
            base: AssetBase::new(name, AssetType::Model),
            mesh_uuids: Vec::new(),
            material_uuids: Vec::new(),
            skeleton_uuid: None,
            node_count: 0,
            skin_names: Vec::new(),
        }
    }

    pub fn add_mesh(&mut self, uuid: &str) {
        self.mesh_uuids.push(uuid.to_string());
    }

    pub fn add_material(&mut self, uuid: &str) {
        self.material_uuids.push(uuid.to_string());
    }

    pub fn set_skeleton(&mut self, uuid: &str) {
        self.skeleton_uuid = Some(uuid.to_string());
    }

    pub fn has_skeleton(&self) -> bool {
        self.skeleton_uuid.is_some()
    }
}

impl RefCounted for ModelAsset {
    fn add_ref(&self) { self.base.ref_count.add_ref(); }
    fn release(&self) { self.base.ref_count.release(); }
    fn get_ref_count(&self) -> u32 { self.base.ref_count.get_ref_count() }
    fn is_last_reference(&self) -> bool { self.base.ref_count.is_last_reference() }
}

impl Model for ModelAsset {
    fn get_name(&self) -> &str { &self.base.name }
    fn is_enabled(&self) -> bool { self.base.loaded }
    fn set_enabled(&mut self, enabled: bool) { self.base.loaded = enabled; }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_base_new() {
        let asset = AssetBase::new("test", AssetType::Texture);
        assert_eq!(asset.get_name(), "test");
        assert_eq!(asset.get_asset_type(), AssetType::Texture);
        assert!(!asset.is_loaded());
    }

    #[test]
    fn test_mesh_asset_new() {
        let mesh = MeshAsset::new("cube");
        assert_eq!(mesh.get_name(), "cube");
        assert_eq!(mesh.get_sub_mesh_count(), 0);
        assert_eq!(mesh.total_vertex_count(), 0);
    }

    #[test]
    fn test_mesh_asset_add_sub_mesh() {
        let mut mesh = MeshAsset::new("cube");
        let sub = SubMeshInfo {
            vertex_count: 24,
            index_count: 36,
            ..SubMeshInfo::default()
        };
        mesh.add_sub_mesh(sub);
        assert_eq!(mesh.get_sub_mesh_count(), 1);
        assert_eq!(mesh.total_vertex_count(), 24);
        assert_eq!(mesh.total_index_count(), 36);
    }

    #[test]
    fn test_mesh_asset_vertex_data() {
        let mut mesh = MeshAsset::new("plane");
        mesh.set_vertex_data(vec![0.0; 100]);
        assert!(mesh.base.is_loaded());
        assert_eq!(mesh.vertex_data.len(), 100);
    }

    #[test]
    fn test_model_asset_new() {
        let model = ModelAsset::new("player");
        assert_eq!(model.get_name(), "player");
        assert!(!model.has_skeleton());
    }

    #[test]
    fn test_model_asset_skeleton() {
        let mut model = ModelAsset::new("character");
        model.set_skeleton("skel-uuid-001");
        assert!(model.has_skeleton());
    }

    #[test]
    fn test_model_asset_meshes_materials() {
        let mut model = ModelAsset::new("vehicle");
        model.add_mesh("mesh-uuid-001");
        model.add_mesh("mesh-uuid-002");
        model.add_material("mat-uuid-001");
        assert_eq!(model.mesh_uuids.len(), 2);
        assert_eq!(model.material_uuids.len(), 1);
    }

    #[test]
    fn test_asset_ref_count() {
        let asset = AssetBase::new("tex", AssetType::Texture);
        assert_eq!(asset.get_ref_count(), 1);
        asset.add_ref();
        assert_eq!(asset.get_ref_count(), 2);
        asset.release();
        assert_eq!(asset.get_ref_count(), 1);
    }
}
