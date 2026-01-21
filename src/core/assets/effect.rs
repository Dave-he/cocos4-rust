use super::asset::Asset;
use crate::base::RefCounted;

pub struct EffectAsset {
    pub name: String,
}

impl EffectAsset {
    pub fn new(name: String) -> Self {
        EffectAsset { name }
    }
}
