use crate::base::RefCounted;

#[derive(Debug, Clone)]
pub struct BakedSkinningModel {
    pub name: String,
}

impl BakedSkinningModel {
    pub fn new(name: String) -> Self {
        BakedSkinningModel { name }
    }
}
