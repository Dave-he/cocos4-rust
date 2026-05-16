use super::deferred_types::{DeferredLight, DeferredSceneData, LightingMode};

pub struct LightingStage {
    pub enabled: bool,
    pub mode: LightingMode,
    pub enable_shadows: bool,
    pub enable_specular: bool,
    light_count: u32,
}

impl LightingStage {
    pub fn new() -> Self {
        Self {
            enabled: true,
            mode: LightingMode::default(),
            enable_shadows: false,
            enable_specular: true,
            light_count: 0,
        }
    }

    pub fn set_mode(&mut self, mode: LightingMode) {
        self.mode = mode;
    }

    pub fn render(&mut self, scene_data: &DeferredSceneData) -> u32 {
        if !self.enabled {
            return 0;
        }
        self.light_count = scene_data.get_light_count() as u32;
        self.light_count
    }

    pub fn process_light(&mut self, light: &DeferredLight) -> bool {
        if light.intensity <= 0.0 {
            return false;
        }
        true
    }

    pub fn cull_lights(&mut self, lights: &[DeferredLight]) -> Vec<u32> {
        lights
            .iter()
            .enumerate()
            .filter(|(_, l)| self.process_light(l))
            .map(|(i, _)| i as u32)
            .collect()
    }

    pub fn get_light_count(&self) -> u32 {
        self.light_count
    }

    pub fn reset(&mut self) {
        self.light_count = 0;
    }
}

impl Default for LightingStage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lighting_stage_new() {
        let stage = LightingStage::new();
        assert!(stage.enabled);
        assert!(stage.enable_specular);
        assert_eq!(stage.mode, LightingMode::Tiled);
    }

    #[test]
    fn test_lighting_stage_render_no_light() {
        let mut stage = LightingStage::new();
        let scene = DeferredSceneData::new();
        let draws = stage.render(&scene);
        assert_eq!(draws, 0);
    }

    #[test]
    fn test_lighting_stage_render_with_lights() {
        let mut stage = LightingStage::new();
        let mut scene = DeferredSceneData::new();
        scene.add_light(DeferredLight::default());
        scene.add_light(DeferredLight::default());
        scene.add_light(DeferredLight::default());
        let draws = stage.render(&scene);
        assert_eq!(draws, 3);
    }

    #[test]
    fn test_light_culling() {
        let mut stage = LightingStage::new();
        let lights = vec![
            DeferredLight {
                intensity: 1.0,
                ..Default::default()
            },
            DeferredLight {
                intensity: 0.0,
                ..Default::default()
            },
            DeferredLight {
                intensity: 2.0,
                ..Default::default()
            },
        ];
        let visible = stage.cull_lights(&lights);
        assert_eq!(visible.len(), 2);
        assert!(visible.contains(&0));
        assert!(visible.contains(&2));
    }

    #[test]
    fn test_lighting_stage_mode() {
        let mut stage = LightingStage::new();
        assert_eq!(stage.mode, LightingMode::Tiled);
        stage.set_mode(LightingMode::Clustered);
        assert_eq!(stage.mode, LightingMode::Clustered);
        stage.set_mode(LightingMode::ForwardPlus);
        assert_eq!(stage.mode, LightingMode::ForwardPlus);
    }
}
