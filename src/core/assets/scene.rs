use super::asset::AssetBase;

#[derive(Debug)]
pub struct SceneAsset {
    pub base: AssetBase,
    pub scene_uuid: Option<String>,
}

impl SceneAsset {
    pub fn new() -> Self {
        SceneAsset {
            base: AssetBase::new(),
            scene_uuid: None,
        }
    }

    pub fn get_scene_uuid(&self) -> Option<&str> {
        self.scene_uuid.as_deref()
    }

    pub fn set_scene_uuid(&mut self, uuid: &str) {
        self.scene_uuid = Some(uuid.to_string());
    }
}

impl Default for SceneAsset {
    fn default() -> Self {
        Self::new()
    }
}
