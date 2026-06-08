use super::layout_graph::LayoutGraph;
use super::render_graph::RenderGraph;
use super::resource_graph::ResourceGraph;
use crate::renderer::pipeline::RenderPipeline;

pub struct NativePipeline {
    pub name: String,
    pub render_graph: RenderGraph,
    pub resource_graph: ResourceGraph,
    pub layout_graph: LayoutGraph,
    pub enabled: bool,
    render_pipeline: Option<RenderPipeline>,
}

impl NativePipeline {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            render_graph: RenderGraph::new(name),
            resource_graph: ResourceGraph::new(),
            layout_graph: LayoutGraph::new(),
            enabled: true,
            render_pipeline: None,
        }
    }

    pub fn set_render_pipeline(&mut self, pipeline: RenderPipeline) {
        self.render_pipeline = Some(pipeline);
    }

    pub fn get_render_pipeline(&self) -> Option<&RenderPipeline> {
        self.render_pipeline.as_ref()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn clear(&mut self) {
        self.render_graph.reset();
        self.resource_graph.clear();
        self.layout_graph.clear();
    }
}

impl Default for NativePipeline {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_pipeline_new() {
        let np = NativePipeline::new("forward");
        assert_eq!(np.name, "forward");
        assert!(np.is_enabled());
    }

    #[test]
    fn test_native_pipeline_enable_disable() {
        let mut np = NativePipeline::new("deferred");
        assert!(np.is_enabled());
        np.disable();
        assert!(!np.is_enabled());
        np.enable();
        assert!(np.is_enabled());
    }

    #[test]
    fn test_native_pipeline_clear() {
        let mut np = NativePipeline::new("test");
        np.render_graph
            .add_pass(super::super::types::PassDesc::default());
        np.clear();
        assert_eq!(np.render_graph.get_node_count(), 0);
    }
}
