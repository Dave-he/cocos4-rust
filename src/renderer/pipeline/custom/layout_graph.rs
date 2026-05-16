use super::types::LayoutDesc;

#[derive(Debug, Clone)]
pub struct DescriptorSetLayoutBinding {
    pub binding: u32,
    pub descriptor_type: u32,
    pub descriptor_count: u32,
    pub stage_flags: u32,
}

#[derive(Debug, Clone)]
pub struct LayoutGraphNode {
    pub id: u32,
    pub desc: LayoutDesc,
    pub bindings: Vec<DescriptorSetLayoutBinding>,
}

#[derive(Debug, Clone)]
pub struct PipelineLayout {
    pub id: u32,
    pub set_layout_ids: Vec<u32>,
    pub push_constant_ranges: Vec<(u32, u32, u32)>,
}

#[derive(Debug, Clone, Default)]
pub struct LayoutGraph {
    pub nodes: Vec<LayoutGraphNode>,
    pub pipeline_layouts: Vec<PipelineLayout>,
    next_id: u32,
}

impl LayoutGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            pipeline_layouts: Vec::new(),
            next_id: 0,
        }
    }

    pub fn create_layout(&mut self, desc: LayoutDesc) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.push(LayoutGraphNode {
            id,
            desc,
            bindings: Vec::new(),
        });
        id
    }

    pub fn add_binding(
        &mut self,
        layout_id: u32,
        binding: u32,
        descriptor_type: u32,
        descriptor_count: u32,
        stage_flags: u32,
    ) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == layout_id) {
            node.bindings.push(DescriptorSetLayoutBinding {
                binding,
                descriptor_type,
                descriptor_count,
                stage_flags,
            });
        }
    }

    pub fn create_pipeline_layout(
        &mut self,
        set_layout_ids: Vec<u32>,
        push_constant_ranges: Vec<(u32, u32, u32)>,
    ) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.pipeline_layouts.push(PipelineLayout {
            id,
            set_layout_ids,
            push_constant_ranges,
        });
        id
    }

    pub fn get_layout(&self, id: u32) -> Option<&LayoutGraphNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn get_pipeline_layout(&self, id: u32) -> Option<&PipelineLayout> {
        self.pipeline_layouts.iter().find(|pl| pl.id == id)
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.pipeline_layouts.clear();
        self.next_id = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_graph_new() {
        let lg = LayoutGraph::new();
        assert_eq!(lg.nodes.len(), 0);
    }

    #[test]
    fn test_create_layout() {
        let mut lg = LayoutGraph::new();
        let id = lg.create_layout(LayoutDesc {
            name: "GlobalSet".into(),
            ..Default::default()
        });
        assert!(lg.get_layout(id).is_some());
    }

    #[test]
    fn test_add_binding() {
        let mut lg = LayoutGraph::new();
        let id = lg.create_layout(LayoutDesc {
            name: "MaterialSet".into(),
            ..Default::default()
        });
        lg.add_binding(id, 0, 0, 1, 1);
        let node = lg.get_layout(id).unwrap();
        assert_eq!(node.bindings.len(), 1);
        assert_eq!(node.bindings[0].binding, 0);
    }

    #[test]
    fn test_create_pipeline_layout() {
        let mut lg = LayoutGraph::new();
        let set0 = lg.create_layout(LayoutDesc::default());
        let set1 = lg.create_layout(LayoutDesc::default());
        let pl_id = lg.create_pipeline_layout(vec![set0, set1], vec![]);
        assert!(lg.get_pipeline_layout(pl_id).is_some());
        let pl = lg.get_pipeline_layout(pl_id).unwrap();
        assert_eq!(pl.set_layout_ids.len(), 2);
    }

    #[test]
    fn test_push_constants() {
        let mut lg = LayoutGraph::new();
        let set0 = lg.create_layout(LayoutDesc::default());
        let pl_id = lg.create_pipeline_layout(vec![set0], vec![(0, 0, 64)]);
        let pl = lg.get_pipeline_layout(pl_id).unwrap();
        assert_eq!(pl.push_constant_ranges.len(), 1);
    }
}
