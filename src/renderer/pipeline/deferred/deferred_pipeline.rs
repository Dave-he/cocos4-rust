use super::bloom_stage::BloomStage;
use super::deferred_scene_data::DeferredSceneDataManager;
use super::deferred_types::DeferredConfig;
use super::gbuffer_stage::GbufferStage;
use super::lighting_stage::LightingStage;
use super::post_process_stage::PostProcessStage;

pub struct DeferredPipeline {
    pub name: String,
    pub enabled: bool,
    pub config: DeferredConfig,
    pub gbuffer_stage: GbufferStage,
    pub lighting_stage: LightingStage,
    pub bloom_stage: BloomStage,
    pub post_process_stage: PostProcessStage,
    pub scene_data: DeferredSceneDataManager,
    initialized: bool,
    frame_draw_calls: u32,
}

impl DeferredPipeline {
    pub fn new(name: &str) -> Self {
        let mut pipeline = Self {
            name: name.to_string(),
            enabled: true,
            config: DeferredConfig::default(),
            gbuffer_stage: GbufferStage::new(),
            lighting_stage: LightingStage::new(),
            bloom_stage: BloomStage::new(),
            post_process_stage: PostProcessStage::new(),
            scene_data: DeferredSceneDataManager::new(),
            initialized: false,
            frame_draw_calls: 0,
        };
        pipeline.bloom_stage.configure(&pipeline.config);
        pipeline
    }

    pub fn initialize(&mut self, width: u32, height: u32) {
        self.gbuffer_stage.initialize(width, height, 1);
        self.config.gbuffer_width = width;
        self.config.gbuffer_height = height;
        self.initialized = true;
    }

    pub fn render(&mut self) -> u32 {
        if !self.enabled || !self.initialized {
            return 0;
        }

        self.frame_draw_calls = 0;

        self.gbuffer_stage.create_gbuffer();

        let gbuffer_draws = self
            .gbuffer_stage
            .render(self.scene_data.get_scene_data());
        self.frame_draw_calls += gbuffer_draws;

        let lighting_draws = self
            .lighting_stage
            .render(self.scene_data.get_scene_data());
        self.frame_draw_calls += lighting_draws;

        let bloom_passes = self.bloom_stage.render(
            self.config.gbuffer_width,
            self.config.gbuffer_height,
        );
        self.frame_draw_calls += bloom_passes;

        let post_passes = self.post_process_stage.render();
        self.frame_draw_calls += post_passes;

        self.frame_draw_calls
    }

    pub fn get_frame_draw_calls(&self) -> u32 {
        self.frame_draw_calls
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn destroy(&mut self) {
        self.gbuffer_stage.clear();
        self.bloom_stage.reset();
        self.lighting_stage.reset();
        self.initialized = false;
    }
}

impl Default for DeferredPipeline {
    fn default() -> Self {
        Self::new("deferred")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deferred_pipeline_new() {
        let pipeline = DeferredPipeline::new("deferred");
        assert_eq!(pipeline.name, "deferred");
        assert!(pipeline.enabled);
        assert!(!pipeline.is_initialized());
    }

    #[test]
    fn test_deferred_pipeline_initialize() {
        let mut pipeline = DeferredPipeline::new("deferred");
        pipeline.initialize(1920, 1080);
        assert!(pipeline.is_initialized());
        assert_eq!(pipeline.config.gbuffer_width, 1920);
        assert_eq!(pipeline.config.gbuffer_height, 1080);
    }

    #[test]
    fn test_deferred_pipeline_render() {
        let mut pipeline = DeferredPipeline::new("deferred");
        pipeline.initialize(1280, 720);
        let draws = pipeline.render();
        assert!(draws >= 6);
        assert_eq!(pipeline.get_frame_draw_calls(), draws);
    }

    #[test]
    fn test_deferred_pipeline_not_initialized() {
        let pipeline = DeferredPipeline::new("deferred");
        assert_eq!(pipeline.get_frame_draw_calls(), 0);
    }

    #[test]
    fn test_deferred_pipeline_enable_disable() {
        let mut pipeline = DeferredPipeline::new("deferred");
        pipeline.initialize(800, 600);
        pipeline.disable();
        assert!(!pipeline.enabled);
        let draws = pipeline.render();
        assert_eq!(draws, 0);
        pipeline.enable();
        assert!(pipeline.enabled);
    }

    #[test]
    fn test_deferred_pipeline_destroy() {
        let mut pipeline = DeferredPipeline::new("deferred");
        pipeline.initialize(800, 600);
        pipeline.destroy();
        assert!(!pipeline.is_initialized());
    }

    #[test]
    fn test_deferred_pipeline_full_flow() {
        let mut pipeline = DeferredPipeline::new("deferred_full");
        pipeline.initialize(1920, 1080);
        pipeline.scene_data.get_scene_data_mut().add_light(
            super::super::deferred_types::DeferredLight::default(),
        );
        let draws = pipeline.render();
        assert!(draws > 0);
        assert_eq!(pipeline.get_frame_draw_calls(), draws);
    }
}
