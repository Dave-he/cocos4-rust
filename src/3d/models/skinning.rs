/****************************************************************************
Rust port of Cocos Creator SkinningModel
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::RefCounted;
use crate::math::Mat4;
use crate::_3d::baked_skinning::Mesh;

#[derive(Debug, Clone)]
pub struct JointInfo {
    pub target_name: String,
    pub bindpose: Mat4,
    pub buffers: Vec<usize>,
    pub indices: Vec<usize>,
}

impl JointInfo {
    pub fn new(target_name: &str, bindpose: Mat4) -> Self {
        JointInfo {
            target_name: target_name.to_string(),
            bindpose,
            buffers: Vec::new(),
            indices: Vec::new(),
        }
    }
}

impl Default for JointInfo {
    fn default() -> Self {
        JointInfo {
            target_name: String::new(),
            bindpose: Mat4::IDENTITY,
            buffers: Vec::new(),
            indices: Vec::new(),
        }
    }
}

pub trait SkinningModel: RefCounted {
    fn get_mesh(&self) -> &Mesh;
    fn bind_skeleton(&mut self, skeleton_root: &str, joint_names: &[String], bind_poses: &[Mat4]);
    fn update_transform(&mut self, stamp: u32);
    fn update_ubos(&mut self, stamp: u32);
    fn get_joints(&self) -> &[JointInfo];
    fn is_real_time_texture_mode(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_joint_info_default() {
        let joint = JointInfo::default();
        assert!(joint.target_name.is_empty());
        assert!(joint.buffers.is_empty());
        assert!(joint.indices.is_empty());
    }

    #[test]
    fn test_joint_info_new() {
        let mat = Mat4::IDENTITY;
        let joint = JointInfo::new("spine", mat);
        assert_eq!(joint.target_name, "spine");
    }
}
