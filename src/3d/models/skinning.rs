use super::baked_skinning::BakedSkinningModel;
use crate::base::RefCounted;

pub trait SkinningModel: RefCounted {
    fn get_mesh(&self) -> &crate::_3d::baked_skinning::Mesh;
}
