use super::deferred_types::DeferredSceneData;

pub struct DeferredSceneDataManager {
    pub scene_data: DeferredSceneData,
    dirty: bool,
}

impl DeferredSceneDataManager {
    pub fn new() -> Self {
        Self {
            scene_data: DeferredSceneData::new(),
            dirty: false,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn get_scene_data(&self) -> &DeferredSceneData {
        &self.scene_data
    }

    pub fn get_scene_data_mut(&mut self) -> &mut DeferredSceneData {
        self.mark_dirty();
        &mut self.scene_data
    }

    pub fn update(&mut self) {
        self.clear_dirty();
    }
}

impl Default for DeferredSceneDataManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::deferred_types::DeferredLight;

    #[test]
    fn test_scene_data_manager_new() {
        let sdm = DeferredSceneDataManager::new();
        assert!(!sdm.is_dirty());
        assert_eq!(sdm.get_scene_data().get_light_count(), 0);
    }

    #[test]
    fn test_scene_data_dirty_flag() {
        let mut sdm = DeferredSceneDataManager::new();
        assert!(!sdm.is_dirty());
        sdm.mark_dirty();
        assert!(sdm.is_dirty());
        sdm.update();
        assert!(!sdm.is_dirty());
    }

    #[test]
    fn test_scene_data_mut_marks_dirty() {
        let mut sdm = DeferredSceneDataManager::new();
        {
            let data = sdm.get_scene_data_mut();
            data.add_light(DeferredLight::default());
        }
        assert!(sdm.is_dirty());
    }
}
