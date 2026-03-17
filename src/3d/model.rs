/****************************************************************************
Rust port of Cocos Creator 3D Model
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::math::{Mat4, Vec3};
use crate::core::geometry::AABB;
use super::mesh::Mesh3D;

#[derive(Debug, Clone)]
pub struct MorphTarget {
    pub name: String,
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
}

#[derive(Debug, Clone)]
pub struct MorphTargetSet {
    pub targets: Vec<MorphTarget>,
    pub weights: Vec<f32>,
}

impl MorphTargetSet {
    pub fn new() -> Self {
        MorphTargetSet {
            targets: Vec::new(),
            weights: Vec::new(),
        }
    }

    pub fn add_target(&mut self, target: MorphTarget, weight: f32) {
        self.targets.push(target);
        self.weights.push(weight);
    }

    pub fn set_weight(&mut self, index: usize, weight: f32) {
        if let Some(w) = self.weights.get_mut(index) {
            *w = weight.clamp(0.0, 1.0);
        }
    }
}

impl Default for MorphTargetSet {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Model3D {
    pub name: String,
    pub model_type: ModelType,
    pub mesh: Option<Mesh3D>,
    pub world_matrix: Mat4,
    pub local_matrix: Mat4,
    pub enabled: bool,
    pub receive_shadows: bool,
    pub cast_shadows: bool,
    pub bounds: AABB,
    pub morph_targets: Option<MorphTargetSet>,
    pub material_ids: Vec<u32>,
}

impl Model3D {
    pub fn new(name: &str) -> Self {
        Model3D {
            name: name.to_string(),
            model_type: ModelType::Default,
            mesh: None,
            world_matrix: Mat4::IDENTITY,
            local_matrix: Mat4::IDENTITY,
            enabled: true,
            receive_shadows: true,
            cast_shadows: true,
            bounds: AABB::default(),
            morph_targets: None,
            material_ids: Vec::new(),
        }
    }

    pub fn set_mesh(&mut self, mesh: Mesh3D) {
        self.bounds = Self::compute_bounds(&mesh);
        self.mesh = Some(mesh);
    }

    fn compute_bounds(mesh: &Mesh3D) -> AABB {
        if mesh.sub_meshes.is_empty() {
            return AABB::default();
        }
        let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
        for sub in &mesh.sub_meshes {
            min.x = min.x.min(sub.min_pos.x);
            min.y = min.y.min(sub.min_pos.y);
            min.z = min.z.min(sub.min_pos.z);
            max.x = max.x.max(sub.max_pos.x);
            max.y = max.y.max(sub.max_pos.y);
            max.z = max.z.max(sub.max_pos.z);
        }
        AABB::from_min_max(min, max)
    }

    pub fn update_transform(&mut self, matrix: Mat4) {
        self.world_matrix = matrix;
    }

    pub fn get_mesh(&self) -> Option<&Mesh3D> {
        self.mesh.as_ref()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn add_material(&mut self, material_id: u32) {
        self.material_ids.push(material_id);
    }
}

impl Default for Model3D {
    fn default() -> Self {
        Self::new("Model")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_new() {
        let aabb = AABB::from_center_extents(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));
        assert!(aabb.contains_point(&Vec3::ZERO));
        assert!(aabb.contains_point(&Vec3::new(0.9, 0.9, 0.9)));
        assert!(!aabb.contains_point(&Vec3::new(1.5, 0.0, 0.0)));
    }

    #[test]
    fn test_aabb_from_min_max() {
        let aabb = AABB::from_min_max(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        assert_eq!(aabb.center, Vec3::ZERO);
        assert_eq!(aabb.half_extents, Vec3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn test_aabb_intersects() {
        let a = AABB::from_center_extents(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));
        let b = AABB::from_center_extents(Vec3::new(1.5, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
        let c = AABB::from_center_extents(Vec3::new(3.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn test_model_new() {
        let model = Model3D::new("TestModel");
        assert_eq!(model.name, "TestModel");
        assert!(model.is_enabled());
        assert!(model.get_mesh().is_none());
    }

    #[test]
    fn test_model_set_mesh() {
        let mut model = Model3D::new("Model");
        let mut mesh = Mesh3D::new("Mesh");
        mesh.add_sub_mesh(super::super::mesh::SubMesh::default());
        model.set_mesh(mesh);
        assert!(model.get_mesh().is_some());
    }

    #[test]
    fn test_morph_target_set() {
        let mut set = MorphTargetSet::new();
        set.add_target(
            MorphTarget {
                name: "smile".to_string(),
                positions: Vec::new(),
                normals: Vec::new(),
            },
            0.0,
        );
        assert_eq!(set.targets.len(), 1);
        set.set_weight(0, 0.8);
        assert_eq!(set.weights[0], 0.8);
        set.set_weight(0, 1.5);
        assert_eq!(set.weights[0], 1.0);
    }
}
