/****************************************************************************
Rust port of Cocos Creator 3D Skinning Model
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::{RefCounted, RefCountedImpl};
use crate::math::{Mat4, Quaternion, Vec3};

#[derive(Debug, Clone)]
pub struct JointInfo {
    pub name: String,
    pub parent_index: i32,
    pub bind_pose_inv: Mat4,
}

impl JointInfo {
    pub fn new(name: &str, parent_index: i32) -> Self {
        JointInfo {
            name: name.to_string(),
            parent_index,
            bind_pose_inv: Mat4::IDENTITY,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SkinningJointState {
    pub local_position: Vec3,
    pub local_rotation: Quaternion,
    pub local_scale: Vec3,
    pub world_matrix: Mat4,
    pub skinning_matrix: Mat4,
}

impl SkinningJointState {
    pub fn new() -> Self {
        SkinningJointState {
            local_position: Vec3::ZERO,
            local_rotation: Quaternion::IDENTITY,
            local_scale: Vec3::ONE,
            world_matrix: Mat4::IDENTITY,
            skinning_matrix: Mat4::IDENTITY,
        }
    }

    pub fn local_to_parent_matrix(&self) -> Mat4 {
        Mat4::from_srt(&self.local_rotation, &self.local_position, &self.local_scale)
    }
}

impl Default for SkinningJointState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct SkinningModel {
    pub name: String,
    pub joints: Vec<JointInfo>,
    pub joint_states: Vec<SkinningJointState>,
    pub skinning_matrices: Vec<Mat4>,
    pub enabled: bool,
    pub mesh_uuid: Option<String>,
    ref_count: RefCountedImpl,
}

impl SkinningModel {
    pub fn new(name: String) -> Self {
        SkinningModel {
            name,
            joints: Vec::new(),
            joint_states: Vec::new(),
            skinning_matrices: Vec::new(),
            enabled: true,
            mesh_uuid: None,
            ref_count: RefCountedImpl::new(),
        }
    }

    pub fn add_joint(&mut self, info: JointInfo) {
        let new_state = SkinningJointState::new();
        self.joints.push(info);
        self.joint_states.push(new_state);
        self.skinning_matrices.push(Mat4::IDENTITY);
    }

    pub fn get_joint_count(&self) -> usize {
        self.joints.len()
    }

    pub fn set_joint_pose(&mut self, index: usize, position: Vec3, rotation: Quaternion, scale: Vec3) {
        if index < self.joint_states.len() {
            self.joint_states[index].local_position = position;
            self.joint_states[index].local_rotation = rotation;
            self.joint_states[index].local_scale = scale;
        }
    }

    pub fn update_world_matrices(&mut self, root_matrix: &Mat4) {
        for i in 0..self.joints.len() {
            let local_mat = self.joint_states[i].local_to_parent_matrix();
            let parent_idx = self.joints[i].parent_index;
            let world_mat = if parent_idx < 0 {
                Mat4::multiply_mat4(root_matrix, &local_mat)
            } else {
                Mat4::multiply_mat4(&self.joint_states[parent_idx as usize].world_matrix, &local_mat)
            };
            self.joint_states[i].world_matrix = world_mat;
        }
    }

    pub fn update_skinning_matrices(&mut self) {
        for i in 0..self.joints.len() {
            self.skinning_matrices[i] = Mat4::multiply_mat4(
                &self.joint_states[i].world_matrix,
                &self.joints[i].bind_pose_inv,
            );
        }
    }

    pub fn update(&mut self, root_matrix: &Mat4) {
        self.update_world_matrices(root_matrix);
        self.update_skinning_matrices();
    }

    pub fn get_skinning_matrix(&self, index: usize) -> Option<&Mat4> {
        self.skinning_matrices.get(index)
    }

    pub fn set_mesh(&mut self, uuid: Option<String>) {
        self.mesh_uuid = uuid;
    }

    pub fn set_bind_pose_inv(&mut self, joint_index: usize, inv: Mat4) {
        if joint_index < self.joints.len() {
            self.joints[joint_index].bind_pose_inv = inv;
        }
    }

    pub fn find_joint_by_name(&self, name: &str) -> Option<usize> {
        self.joints.iter().position(|j| j.name == name)
    }
}

impl RefCounted for SkinningModel {
    fn add_ref(&self) { self.ref_count.add_ref(); }
    fn release(&self) { self.ref_count.release(); }
    fn get_ref_count(&self) -> u32 { self.ref_count.get_ref_count() }
    fn is_last_reference(&self) -> bool { self.ref_count.is_last_reference() }
}

#[derive(Debug)]
pub struct BakedSkinningModel {
    pub name: String,
    pub joints: Vec<JointInfo>,
    pub baked_matrices: Vec<Vec<Mat4>>,
    pub current_frame: usize,
    pub frame_count: usize,
    pub enabled: bool,
    ref_count: RefCountedImpl,
}

impl BakedSkinningModel {
    pub fn new(name: String) -> Self {
        BakedSkinningModel {
            name,
            joints: Vec::new(),
            baked_matrices: Vec::new(),
            current_frame: 0,
            frame_count: 0,
            enabled: true,
            ref_count: RefCountedImpl::new(),
        }
    }

    pub fn add_joint(&mut self, info: JointInfo) {
        self.joints.push(info);
    }

    pub fn bake_frame(&mut self, frame_matrices: Vec<Mat4>) {
        if self.baked_matrices.is_empty() {
            self.baked_matrices = vec![Vec::new(); self.joints.len()];
        }
        for (i, mat) in frame_matrices.into_iter().enumerate() {
            if i < self.joints.len() {
                self.baked_matrices[i].push(mat);
            }
        }
        if !self.joints.is_empty() {
            self.frame_count = self.baked_matrices[0].len();
        }
    }

    pub fn get_frame_count(&self) -> usize {
        self.frame_count
    }

    pub fn set_frame(&mut self, frame: usize) {
        if frame < self.frame_count {
            self.current_frame = frame;
        }
    }

    pub fn get_current_skinning_matrix(&self, joint_index: usize) -> Option<&Mat4> {
        self.baked_matrices
            .get(joint_index)
            .and_then(|frames| frames.get(self.current_frame))
    }
}

impl RefCounted for BakedSkinningModel {
    fn add_ref(&self) { self.ref_count.add_ref(); }
    fn release(&self) { self.ref_count.release(); }
    fn get_ref_count(&self) -> u32 { self.ref_count.get_ref_count() }
    fn is_last_reference(&self) -> bool { self.ref_count.is_last_reference() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_model() -> SkinningModel {
        let mut model = SkinningModel::new("character".to_string());
        model.add_joint(JointInfo::new("root", -1));
        model.add_joint(JointInfo::new("spine", 0));
        model.add_joint(JointInfo::new("head", 1));
        model
    }

    #[test]
    fn test_skinning_model_new() {
        let model = make_model();
        assert_eq!(model.name, "character");
        assert_eq!(model.get_joint_count(), 3);
    }

    #[test]
    fn test_skinning_model_find_joint() {
        let model = make_model();
        assert_eq!(model.find_joint_by_name("spine"), Some(1));
        assert_eq!(model.find_joint_by_name("missing"), None);
    }

    #[test]
    fn test_skinning_model_set_pose() {
        let mut model = make_model();
        model.set_joint_pose(0, Vec3::new(0.0, 1.0, 0.0), Quaternion::IDENTITY, Vec3::ONE);
        assert_eq!(model.joint_states[0].local_position, Vec3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn test_skinning_model_update() {
        let mut model = make_model();
        model.update(&Mat4::IDENTITY);
        let mat = model.get_skinning_matrix(0);
        assert!(mat.is_some());
    }

    #[test]
    fn test_baked_skinning_model() {
        let mut model = BakedSkinningModel::new("baked".to_string());
        model.add_joint(JointInfo::new("root", -1));
        model.bake_frame(vec![Mat4::IDENTITY]);
        model.bake_frame(vec![Mat4::IDENTITY]);
        assert_eq!(model.get_frame_count(), 2);
        model.set_frame(1);
        assert_eq!(model.current_frame, 1);
        let mat = model.get_current_skinning_matrix(0);
        assert!(mat.is_some());
    }

    #[test]
    fn test_joint_info_parent() {
        let j = JointInfo::new("arm", 2);
        assert_eq!(j.parent_index, 2);
        assert_eq!(j.name, "arm");
    }

    #[test]
    fn test_skinning_model_ref_count() {
        let model = SkinningModel::new("test".to_string());
        assert_eq!(model.get_ref_count(), 1);
        model.add_ref();
        assert_eq!(model.get_ref_count(), 2);
    }
}
