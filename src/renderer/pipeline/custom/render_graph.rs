use super::types::{PassDesc, PassKind, ResourceDesc, ResourceKind, SubpassDesc};

#[derive(Debug, Clone)]
pub struct RenderGraphNode {
    pub id: u32,
    pub pass: PassDesc,
    pub enabled: bool,
    pub ref_count: u32,
    pub dependencies: Vec<u32>,
    pub subpasses: Vec<SubpassDesc>,
}

impl RenderGraphNode {
    pub fn new(id: u32, pass: PassDesc) -> Self {
        Self {
            id,
            pass,
            enabled: true,
            ref_count: 0,
            dependencies: Vec::new(),
            subpasses: Vec::new(),
        }
    }

    pub fn add_dependency(&mut self, node_id: u32) {
        if !self.dependencies.contains(&node_id) {
            self.dependencies.push(node_id);
        }
    }

    pub fn remove_dependency(&mut self, node_id: u32) {
        self.dependencies.retain(|&id| id != node_id);
    }
}

#[derive(Debug, Clone)]
pub struct GraphResource {
    pub id: u32,
    pub desc: ResourceDesc,
    pub is_imported: bool,
    pub producer: Option<u32>,
    pub consumers: Vec<u32>,
    pub first_use: Option<u32>,
    pub last_use: Option<u32>,
    pub version: u32,
}

impl GraphResource {
    pub fn new(id: u32, desc: ResourceDesc) -> Self {
        Self {
            id,
            desc,
            is_imported: false,
            producer: None,
            consumers: Vec::new(),
            first_use: None,
            last_use: None,
            version: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderGraph {
    pub name: String,
    pub nodes: Vec<RenderGraphNode>,
    pub resources: Vec<GraphResource>,
    pub compiled: bool,
    pub settings: super::types::RenderGraphSettings,
    next_resource_id: u32,
    next_node_id: u32,
}

impl RenderGraph {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            nodes: Vec::new(),
            resources: Vec::new(),
            compiled: false,
            settings: super::types::RenderGraphSettings::default(),
            next_resource_id: 0,
            next_node_id: 0,
        }
    }

    pub fn add_pass(&mut self, pass: PassDesc) -> u32 {
        let id = self.next_node_id;
        self.next_node_id += 1;
        self.nodes.push(RenderGraphNode::new(id, pass));
        id
    }

    pub fn add_resource(&mut self, desc: ResourceDesc) -> u32 {
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        self.resources.push(GraphResource::new(id, desc));
        id
    }

    pub fn get_resource(&self, name: &str) -> Option<&GraphResource> {
        self.resources.iter().find(|r| r.desc.name == name)
    }

    pub fn get_resource_mut(&mut self, name: &str) -> Option<&mut GraphResource> {
        self.resources.iter_mut().find(|r| r.desc.name == name)
    }

    pub fn get_resource_by_id(&self, id: u32) -> Option<&GraphResource> {
        self.resources.iter().find(|r| r.id == id)
    }

    pub fn get_pass(&self, name: &str) -> Option<&RenderGraphNode> {
        self.nodes.iter().find(|n| n.pass.name == name)
    }

    pub fn get_pass_mut(&mut self, name: &str) -> Option<&mut RenderGraphNode> {
        self.nodes.iter_mut().find(|n| n.pass.name == name)
    }

    pub fn get_pass_by_id(&self, id: u32) -> Option<&RenderGraphNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn add_pass_dependency(&mut self, from: u32, to: u32) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == from) {
            node.add_dependency(to);
        }
    }

    pub fn remove_pass(&mut self, id: u32) {
        self.nodes.retain(|n| n.id != id);
        for node in &mut self.nodes {
            node.remove_dependency(id);
        }
    }

    pub fn set_resource_producer(&mut self, resource_id: u32, pass_id: u32) {
        if let Some(res) = self.resources.iter_mut().find(|r| r.id == resource_id) {
            res.producer = Some(pass_id);
        }
    }

    pub fn add_resource_consumer(&mut self, resource_id: u32, pass_id: u32) {
        if let Some(res) = self.resources.iter_mut().find(|r| r.id == resource_id) {
            if !res.consumers.contains(&pass_id) {
                res.consumers.push(pass_id);
            }
        }
    }

    pub fn compile(&mut self) -> bool {
        self.update_resource_lifetimes();
        self.compiled = true;
        true
    }

    fn update_resource_lifetimes(&mut self) {
        for res in &mut self.resources {
            if let Some(prod_id) = res.producer {
                res.first_use = Some(prod_id);
                res.last_use = Some(prod_id);
            }
            let consumers: Vec<u32> = res.consumers.clone();
            for cid in &consumers {
                let cid_val = *cid;
                if res.first_use.is_none() || cid_val < res.first_use.unwrap() {
                    res.first_use = Some(cid_val);
                }
                if res.last_use.is_none() || cid_val > res.last_use.unwrap() {
                    res.last_use = Some(cid_val);
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.nodes.clear();
        self.resources.clear();
        self.compiled = false;
        self.next_resource_id = 0;
        self.next_node_id = 0;
    }

    pub fn get_node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn get_resource_count(&self) -> usize {
        self.resources.len()
    }

    pub fn is_compiled(&self) -> bool {
        self.compiled
    }

    pub fn cull_disabled(&mut self) -> usize {
        let before = self.nodes.len();
        let disabled: Vec<u32> = self
            .nodes
            .iter()
            .filter(|n| !n.enabled)
            .map(|n| n.id)
            .collect();
        for id in disabled {
            self.remove_pass(id);
        }
        before - self.nodes.len()
    }

    pub fn get_pass_execution_order(&self) -> Vec<u32> {
        self.nodes.iter().map(|n| n.id).collect()
    }

    pub fn export_graphviz(&self) -> String {
        let mut s = format!("digraph {} {{\n", self.name);
        s.push_str("  rankdir=TB;\n");
        s.push_str("  node [shape=box];\n");
        for node in &self.nodes {
            let label = format!("{} [{}]", node.pass.name, node.id);
            s.push_str(&format!("  n{} [label=\"{}\"];\n", node.id, label));
        }
        for node in &self.nodes {
            for dep in &node.dependencies {
                s.push_str(&format!("  n{} -> n{};\n", node.id, dep));
            }
        }
        s.push_str("}\n");
        s
    }
}

impl Default for RenderGraph {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pass(name: &str, kind: PassKind) -> PassDesc {
        PassDesc {
            name: name.to_string(),
            kind,
            ..Default::default()
        }
    }

    fn create_test_resource(name: &str, kind: ResourceKind) -> ResourceDesc {
        ResourceDesc {
            name: name.to_string(),
            kind,
            width: 1920,
            height: 1080,
            format: 0,
            ..Default::default()
        }
    }

    #[test]
    fn test_render_graph_new() {
        let graph = RenderGraph::new("TestGraph");
        assert_eq!(graph.name, "TestGraph");
        assert_eq!(graph.get_node_count(), 0);
        assert_eq!(graph.get_resource_count(), 0);
        assert!(!graph.is_compiled());
    }

    #[test]
    fn test_render_graph_add_pass() {
        let mut graph = RenderGraph::new("TestGraph");
        let id = graph.add_pass(create_test_pass("gbuffer", PassKind::Raster));
        assert_eq!(id, 0);
        assert_eq!(graph.get_node_count(), 1);
        let id2 = graph.add_pass(create_test_pass("lighting", PassKind::Raster));
        assert_eq!(id2, 1);
        assert_eq!(graph.get_node_count(), 2);
    }

    #[test]
    fn test_render_graph_add_resource() {
        let mut graph = RenderGraph::new("TestGraph");
        let id = graph.add_resource(create_test_resource("albedo", ResourceKind::ManagedTexture));
        assert_eq!(id, 0);
        assert_eq!(graph.get_resource_count(), 1);
    }

    #[test]
    fn test_render_graph_compile() {
        let mut graph = RenderGraph::new("TestGraph");
        let res = graph.add_resource(create_test_resource("rt0", ResourceKind::ManagedTexture));
        let pass = graph.add_pass(create_test_pass("draw", PassKind::Raster));
        graph.set_resource_producer(res, pass);
        graph.add_resource_consumer(res, pass);
        assert!(graph.compile());
        assert!(graph.is_compiled());
        let r = graph.get_resource_by_id(res).unwrap();
        assert_eq!(r.first_use, Some(pass));
        assert_eq!(r.last_use, Some(pass));
    }

    #[test]
    fn test_render_graph_pass_dependency() {
        let mut graph = RenderGraph::new("TestGraph");
        let pass0 = graph.add_pass(create_test_pass("shadow", PassKind::Raster));
        let pass1 = graph.add_pass(create_test_pass("gbuffer", PassKind::Raster));
        let pass2 = graph.add_pass(create_test_pass("lighting", PassKind::Raster));
        graph.add_pass_dependency(pass2, pass1);
        graph.add_pass_dependency(pass1, pass0);
        graph.compile();
        let node = graph.get_pass_by_id(pass2).unwrap();
        assert!(node.dependencies.contains(&pass1));
    }

    #[test]
    fn test_render_graph_remove_pass() {
        let mut graph = RenderGraph::new("TestGraph");
        let pass0 = graph.add_pass(create_test_pass("p0", PassKind::Raster));
        let pass1 = graph.add_pass(create_test_pass("p1", PassKind::Raster));
        graph.add_pass_dependency(pass1, pass0);
        graph.remove_pass(pass0);
        assert_eq!(graph.get_node_count(), 1);
        let node = graph.get_pass_by_id(pass1).unwrap();
        assert!(!node.dependencies.contains(&pass0));
    }

    #[test]
    fn test_render_graph_reset() {
        let mut graph = RenderGraph::new("TestGraph");
        graph.add_pass(create_test_pass("p0", PassKind::Raster));
        graph.add_resource(create_test_resource("r0", ResourceKind::ManagedTexture));
        graph.reset();
        assert_eq!(graph.get_node_count(), 0);
        assert_eq!(graph.get_resource_count(), 0);
    }

    #[test]
    fn test_render_graph_cull_disabled() {
        let mut graph = RenderGraph::new("TestGraph");
        graph.add_pass(create_test_pass("p0", PassKind::Raster));
        let pass1 = graph.add_pass(create_test_pass("p1", PassKind::Raster));
        graph.add_pass(create_test_pass("p2", PassKind::Raster));
        if let Some(node) = graph.nodes.iter_mut().find(|n| n.id == pass1) {
            node.enabled = false;
        }
        let removed = graph.cull_disabled();
        assert_eq!(removed, 1);
        assert_eq!(graph.get_node_count(), 2);
    }

    #[test]
    fn test_render_graph_export_graphviz() {
        let mut graph = RenderGraph::new("DeferredPipeline");
        let pass0 = graph.add_pass(create_test_pass("shadow", PassKind::Raster));
        let pass1 = graph.add_pass(create_test_pass("gbuffer", PassKind::Raster));
        graph.add_pass_dependency(pass1, pass0);
        let result = graph.export_graphviz();
        assert!(result.contains("DeferredPipeline"));
        assert!(result.contains("shadow"));
        assert!(result.contains("gbuffer"));
    }

    #[test]
    fn test_render_graph_complex_pipeline() {
        let mut graph = RenderGraph::new("Pipeline");
        let rt_albedo = graph.add_resource(create_test_resource("albedo", ResourceKind::ManagedTexture));
        let rt_normal = graph.add_resource(create_test_resource("normal", ResourceKind::ManagedTexture));
        let rt_depth = graph.add_resource(create_test_resource("depth", ResourceKind::ManagedTexture));
        let rt_output = graph.add_resource(create_test_resource("output", ResourceKind::ManagedTexture));

        let gbuffer = graph.add_pass(create_test_pass("gbuffer", PassKind::Raster));
        graph.set_resource_producer(rt_albedo, gbuffer);
        graph.set_resource_producer(rt_normal, gbuffer);
        graph.set_resource_producer(rt_depth, gbuffer);

        let lighting = graph.add_pass(create_test_pass("lighting", PassKind::Raster));
        graph.add_resource_consumer(rt_albedo, lighting);
        graph.add_resource_consumer(rt_normal, lighting);
        graph.add_resource_consumer(rt_depth, lighting);
        graph.set_resource_producer(rt_output, lighting);
        graph.add_pass_dependency(lighting, gbuffer);

        graph.compile();
        assert_eq!(graph.get_node_count(), 2);
        assert_eq!(graph.get_resource_count(), 4);
        assert!(graph.is_compiled());
    }
}
