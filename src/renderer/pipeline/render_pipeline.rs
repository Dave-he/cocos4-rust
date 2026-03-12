use super::render_flow::RenderFlow;
use super::shadow::ShadowsInfo;
use super::states::PipelineStateManager;

#[derive(Debug)]
pub struct RenderPipelineInfo {
    pub name: String,
}

#[derive(Debug)]
pub struct RenderPipeline {
    pub name: String,
    pub flows: Vec<RenderFlow>,
    pub shadows: ShadowsInfo,
    pub state_manager: PipelineStateManager,
    pub initialized: bool,
    pub width: u32,
    pub height: u32,
}

impl RenderPipeline {
    pub fn new() -> Self {
        RenderPipeline {
            name: String::new(),
            flows: Vec::new(),
            shadows: ShadowsInfo::default(),
            state_manager: PipelineStateManager::new(),
            initialized: false,
            width: 0,
            height: 0,
        }
    }

    pub fn initialize(&mut self, info: RenderPipelineInfo) -> bool {
        self.name = info.name;
        self.initialized = true;
        true
    }

    pub fn destroy(&mut self) {
        for flow in &mut self.flows {
            flow.destroy();
        }
        self.flows.clear();
        self.initialized = false;
    }

    pub fn activate(&mut self) {
        for flow in &mut self.flows {
            flow.activate();
        }
    }

    pub fn on_global_pipeline_state_changed(&mut self) {
        self.state_manager.set_shadow_enabled(self.shadows.enabled);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn add_flow(&mut self, flow: RenderFlow) {
        let mut flows = std::mem::take(&mut self.flows);
        flows.push(flow);
        flows.sort_by_key(|f| f.priority);
        self.flows = flows;
    }

    pub fn get_flow(&self, name: &str) -> Option<&RenderFlow> {
        self.flows.iter().find(|f| f.name == name)
    }

    pub fn get_flow_mut(&mut self, name: &str) -> Option<&mut RenderFlow> {
        self.flows.iter_mut().find(|f| f.name == name)
    }

    pub fn get_shadows(&self) -> &ShadowsInfo {
        &self.shadows
    }

    pub fn get_shadows_mut(&mut self) -> &mut ShadowsInfo {
        &mut self.shadows
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for RenderPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_pipeline_new() {
        let pipeline = RenderPipeline::new();
        assert!(!pipeline.is_initialized());
        assert!(pipeline.flows.is_empty());
    }

    #[test]
    fn test_render_pipeline_initialize() {
        let mut pipeline = RenderPipeline::new();
        let result = pipeline.initialize(RenderPipelineInfo {
            name: "forward".to_string(),
        });
        assert!(result);
        assert!(pipeline.is_initialized());
        assert_eq!(pipeline.name, "forward");
    }

    #[test]
    fn test_render_pipeline_add_flow() {
        let mut pipeline = RenderPipeline::new();
        pipeline.add_flow(RenderFlow::new("shadows", 10));
        pipeline.add_flow(RenderFlow::new("forward", 0));
        assert_eq!(pipeline.flows.len(), 2);
        assert_eq!(pipeline.flows[0].name, "forward");
        assert_eq!(pipeline.flows[1].name, "shadows");
    }

    #[test]
    fn test_render_pipeline_resize() {
        let mut pipeline = RenderPipeline::new();
        pipeline.resize(1920, 1080);
        assert_eq!(pipeline.width, 1920);
        assert_eq!(pipeline.height, 1080);
    }

    #[test]
    fn test_render_pipeline_shadows() {
        let mut pipeline = RenderPipeline::new();
        pipeline.get_shadows_mut().set_enabled(true);
        assert!(pipeline.get_shadows().enabled);
    }
}
