use crate::base::RefCounted;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MorphTarget {
    Positions = 0,
    Normals = 1,
    Tangents = 2,
}

pub struct MorphTargetDescription {
    pub targets: Vec<String>,
    pub weights: Vec<f32>,
}

pub trait MorphModel: RefCounted {
    fn get_mesh(&self) -> &crate::_3d::baked_skinning::Mesh;
}
