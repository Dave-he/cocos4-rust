#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostProcess {
    ToneMapping,
    GammaCorrection,
    FXAA,
    ColorGrading,
    Vignette,
}

pub struct PostProcessStage {
    pub enabled: bool,
    pub active_effects: Vec<PostProcess>,
    pub tonemap_exposure: f32,
    pub gamma: f32,
    pub vignette_intensity: f32,
}

impl PostProcessStage {
    pub fn new() -> Self {
        Self {
            enabled: true,
            active_effects: vec![
                PostProcess::ToneMapping,
                PostProcess::GammaCorrection,
            ],
            tonemap_exposure: 1.0,
            gamma: 2.2,
            vignette_intensity: 0.0,
        }
    }

    pub fn add_effect(&mut self, effect: PostProcess) {
        if !self.active_effects.contains(&effect) {
            self.active_effects.push(effect);
        }
    }

    pub fn remove_effect(&mut self, effect: PostProcess) {
        self.active_effects.retain(|e| *e != effect);
    }

    pub fn has_effect(&self, effect: PostProcess) -> bool {
        self.active_effects.contains(&effect)
    }

    pub fn render(&mut self) -> u32 {
        if !self.enabled {
            return 0;
        }
        self.active_effects.len() as u32
    }

    pub fn get_effect_count(&self) -> usize {
        self.active_effects.len()
    }

    pub fn clear(&mut self) {
        self.active_effects.clear();
    }
}

impl Default for PostProcessStage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_process_new() {
        let pp = PostProcessStage::new();
        assert!(pp.enabled);
        assert!(pp.has_effect(PostProcess::ToneMapping));
        assert!(pp.has_effect(PostProcess::GammaCorrection));
    }

    #[test]
    fn test_post_process_add_remove() {
        let mut pp = PostProcessStage::new();
        pp.add_effect(PostProcess::FXAA);
        assert!(pp.has_effect(PostProcess::FXAA));
        assert_eq!(pp.get_effect_count(), 3);
        pp.remove_effect(PostProcess::FXAA);
        assert!(!pp.has_effect(PostProcess::FXAA));
        assert_eq!(pp.get_effect_count(), 2);
    }

    #[test]
    fn test_post_process_render() {
        let mut pp = PostProcessStage::new();
        let passes = pp.render();
        assert_eq!(passes, 2);
    }

    #[test]
    fn test_post_process_disabled() {
        let mut pp = PostProcessStage::new();
        pp.enabled = false;
        assert_eq!(pp.render(), 0);
    }

    #[test]
    fn test_post_process_all_effects() {
        let mut pp = PostProcessStage::new();
        pp.add_effect(PostProcess::FXAA);
        pp.add_effect(PostProcess::ColorGrading);
        pp.add_effect(PostProcess::Vignette);
        assert_eq!(pp.get_effect_count(), 5);
    }
}
