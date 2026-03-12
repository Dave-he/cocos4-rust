use crate::scene::Camera;
use crate::scene::Model;

#[derive(Debug, Default)]
pub struct SceneCulling {
    pub visible_models: Vec<u64>,
}

impl SceneCulling {
    pub fn new() -> Self {
        SceneCulling {
            visible_models: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.visible_models.clear();
    }

    pub fn cull_models(&mut self, camera: &Camera, models: &[Model]) {
        self.visible_models.clear();

        for model in models {
            if !model.enabled {
                continue;
            }

            if model.visibility & camera.visibility == 0 {
                continue;
            }

            self.visible_models.push(model.model_id);
        }
    }

    pub fn get_visible_model_count(&self) -> usize {
        self.visible_models.len()
    }

    pub fn is_visible(&self, model_id: u64) -> bool {
        self.visible_models.contains(&model_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_culling_new() {
        let culling = SceneCulling::new();
        assert_eq!(culling.get_visible_model_count(), 0);
    }

    #[test]
    fn test_scene_culling_clear() {
        let mut culling = SceneCulling::new();
        culling.visible_models.push(1);
        culling.clear();
        assert_eq!(culling.get_visible_model_count(), 0);
    }

    #[test]
    fn test_scene_culling_is_visible() {
        let mut culling = SceneCulling::new();
        culling.visible_models.push(42);
        assert!(culling.is_visible(42));
        assert!(!culling.is_visible(99));
    }
}
