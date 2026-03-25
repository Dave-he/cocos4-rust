/****************************************************************************
Rust port of Cocos Creator MorphModel
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::RefCounted;
use crate::_3d::baked_skinning::Mesh;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MorphTarget {
    Positions = 0,
    Normals = 1,
    Tangents = 2,
}

#[derive(Debug, Clone)]
pub struct MorphTargetDescription {
    pub targets: Vec<String>,
    pub weights: Vec<f32>,
}

impl MorphTargetDescription {
    pub fn new() -> Self {
        MorphTargetDescription {
            targets: Vec::new(),
            weights: Vec::new(),
        }
    }
}

impl Default for MorphTargetDescription {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct MacroPatch {
    pub name: String,
    pub value: bool,
}

impl MacroPatch {
    pub fn new(name: &str, value: bool) -> Self {
        MacroPatch {
            name: name.to_string(),
            value,
        }
    }
}

pub trait MorphModel: RefCounted {
    fn get_mesh(&self) -> &Mesh;
    fn get_macro_patches(&self, sub_model_index: u32) -> Vec<MacroPatch>;
    fn init_sub_model(&mut self, idx: u32, mesh: &Mesh);
    fn destroy(&mut self);
    fn set_sub_model_material(&mut self, idx: u32, material_name: &str);
    fn set_morph_rendering(&mut self, rendering_instance_id: u32);
    fn update_local_descriptors(&mut self, sub_model_index: u32, descriptor_set_id: u32);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_morph_target_description_default() {
        let desc = MorphTargetDescription::default();
        assert!(desc.targets.is_empty());
        assert!(desc.weights.is_empty());
    }

    #[test]
    fn test_macro_patch_new() {
        let patch = MacroPatch::new("CC_USE_MORPH", true);
        assert_eq!(patch.name, "CC_USE_MORPH");
        assert!(patch.value);
    }

    #[test]
    fn test_morph_target_variants() {
        assert_ne!(MorphTarget::Positions as u32, MorphTarget::Normals as u32);
        assert_ne!(MorphTarget::Normals as u32, MorphTarget::Tangents as u32);
    }
}
