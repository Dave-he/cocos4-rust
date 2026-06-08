use super::layout_graph::LayoutGraph;
use super::render_graph::RenderGraph;
use super::resource_graph::ResourceGraph;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilePhase {
    Sort,
    Cull,
    ComputeLifetimes,
    Merge,
    GenerateBarriers,
    Done,
}

impl Default for CompilePhase {
    fn default() -> Self {
        Self::Sort
    }
}

#[derive(Debug, Clone)]
pub struct FrameGraphDispatcher {
    pub render_graph: RenderGraph,
    pub resource_graph: ResourceGraph,
    pub layout_graph: LayoutGraph,
    pub compiled: bool,
    pub phase: CompilePhase,
}

impl FrameGraphDispatcher {
    pub fn new(name: &str) -> Self {
        Self {
            render_graph: RenderGraph::new(name),
            resource_graph: ResourceGraph::new(),
            layout_graph: LayoutGraph::new(),
            compiled: false,
            phase: CompilePhase::Sort,
        }
    }

    pub fn compile(&mut self) -> bool {
        self.phase = CompilePhase::Sort;

        if !self.render_graph.is_compiled() {
            self.render_graph.compile();
        }
        self.phase = CompilePhase::Cull;

        self.render_graph.cull_disabled();
        self.phase = CompilePhase::ComputeLifetimes;

        self.allocate_transient_resources();
        self.phase = CompilePhase::Merge;

        self.generate_barriers();
        self.phase = CompilePhase::Done;

        self.compiled = true;
        true
    }

    fn allocate_transient_resources(&mut self) {
        for res in &self.render_graph.resources {
            if !res.desc.name.is_empty() && res.producer.is_some() {
                if let None = self.resource_graph.get_managed(res.id) {
                    self.resource_graph.create_managed(res.desc.clone());
                    if res.last_use.is_some() {
                        self.resource_graph.allocate_managed(res.id);
                    }
                }
            }
        }
    }

    fn generate_barriers(&mut self) {}

    pub fn execute(&self) -> Result<(), String> {
        if !self.compiled {
            return Err("FrameGraph not compiled".to_string());
        }
        Ok(())
    }

    pub fn reset(&mut self) {
        self.render_graph.reset();
        self.resource_graph.clear();
        self.layout_graph.clear();
        self.compiled = false;
        self.phase = CompilePhase::Sort;
    }

    pub fn get_barrier_count(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::*;
    use super::*;

    #[test]
    fn test_dispatcher_new() {
        let dispatcher = FrameGraphDispatcher::new("test");
        assert!(!dispatcher.compiled);
        assert_eq!(dispatcher.phase, CompilePhase::Sort);
    }

    #[test]
    fn test_dispatcher_compile() {
        let mut dispatcher = FrameGraphDispatcher::new("test");
        dispatcher.render_graph.add_pass(PassDesc {
            name: "p0".into(),
            kind: PassKind::Raster,
            ..Default::default()
        });
        assert!(dispatcher.compile());
        assert!(dispatcher.compiled);
        assert_eq!(dispatcher.phase, CompilePhase::Done);
    }

    #[test]
    fn test_dispatcher_execute_not_compiled() {
        let dispatcher = FrameGraphDispatcher::new("test");
        assert!(dispatcher.execute().is_err());
    }

    #[test]
    fn test_dispatcher_execute_compiled() {
        let mut dispatcher = FrameGraphDispatcher::new("test");
        dispatcher.compile();
        assert!(dispatcher.execute().is_ok());
    }

    #[test]
    fn test_dispatcher_reset() {
        let mut dispatcher = FrameGraphDispatcher::new("test");
        dispatcher.render_graph.add_pass(PassDesc::default());
        dispatcher
            .resource_graph
            .create_managed(ResourceDesc::default());
        dispatcher.compile();
        dispatcher.reset();
        assert!(!dispatcher.compiled);
        assert_eq!(dispatcher.render_graph.get_node_count(), 0);
        assert_eq!(dispatcher.resource_graph.managed.len(), 0);
    }

    #[test]
    fn test_dispatcher_with_resources() {
        let mut dispatcher = FrameGraphDispatcher::new("pipeline");
        let res = dispatcher.render_graph.add_resource(ResourceDesc {
            name: "rt".into(),
            kind: ResourceKind::ManagedTexture,
            width: 1920,
            height: 1080,
            ..Default::default()
        });
        let pass = dispatcher.render_graph.add_pass(PassDesc {
            name: "draw".into(),
            kind: PassKind::Raster,
            ..Default::default()
        });
        dispatcher.render_graph.set_resource_producer(res, pass);
        dispatcher.compile();
        assert!(dispatcher.compiled);
    }
}
