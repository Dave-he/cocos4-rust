/****************************************************************************
Rust port of Cocos Creator Frame Graph System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

pub mod blackboard;
pub mod callback_pass;
pub mod device_pass;
pub mod pass;
pub mod pass_insert_point;
pub mod pass_node_builder;
pub mod resource;

pub use blackboard::*;
pub use callback_pass::*;
pub use device_pass::*;
pub use pass::*;
pub use pass_insert_point::*;
pub use pass_node_builder::*;
pub use resource::*;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ResourceNode {
    pub id: u32,
    pub name: String,
    pub version: u8,
    pub ref_count: u32,
    pub first_pass: u32,
    pub last_pass: u32,
    pub writer_pass_id: u32,
    pub reader_count: u32,
    pub virtual_resource_index: u32,
}

impl ResourceNode {
    pub fn new(id: u32, name: &str) -> Self {
        ResourceNode {
            id,
            name: name.to_string(),
            version: 0,
            ref_count: 0,
            first_pass: u32::MAX,
            last_pass: 0,
            writer_pass_id: u32::MAX,
            reader_count: 0,
            virtual_resource_index: id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualResourceKind {
    Buffer,
    Texture,
}

#[derive(Debug, Clone)]
pub struct VirtualResource {
    pub name: String,
    pub id: u32,
    pub kind: VirtualResourceKind,
    pub external: bool,
    pub never_loaded: bool,
    pub never_stored: bool,
    pub memoryless: bool,
    pub memoryless_msaa: bool,
    pub writer_pass_id: u32,
    pub first_use_pass_id: u32,
    pub last_use_pass_id: u32,
    pub ref_count: u32,
    pub writer_count: u16,
    pub version: u8,
}

impl VirtualResource {
    pub fn new_texture(id: u32, name: &str, external: bool) -> Self {
        VirtualResource {
            name: name.to_string(),
            id,
            kind: VirtualResourceKind::Texture,
            external,
            never_loaded: !external,
            never_stored: true,
            memoryless: false,
            memoryless_msaa: false,
            writer_pass_id: u32::MAX,
            first_use_pass_id: u32::MAX,
            last_use_pass_id: 0,
            ref_count: 0,
            writer_count: 0,
            version: 0,
        }
    }

    pub fn new_buffer(id: u32, name: &str, external: bool) -> Self {
        VirtualResource {
            name: name.to_string(),
            id,
            kind: VirtualResourceKind::Buffer,
            external,
            never_loaded: !external,
            never_stored: true,
            memoryless: false,
            memoryless_msaa: false,
            writer_pass_id: u32::MAX,
            first_use_pass_id: u32::MAX,
            last_use_pass_id: 0,
            ref_count: 0,
            writer_count: 0,
            version: 0,
        }
    }

    pub fn update_lifetime(&mut self, pass_id: u32) {
        if pass_id < self.first_use_pass_id {
            self.first_use_pass_id = pass_id;
        }
        if pass_id > self.last_use_pass_id {
            self.last_use_pass_id = pass_id;
        }
        self.ref_count += 1;
    }

    pub fn new_version(&mut self) -> u8 {
        self.version += 1;
        self.version
    }

    pub fn is_imported(&self) -> bool {
        self.external
    }
}

pub struct FrameGraph {
    pass_nodes: Vec<PassNode>,
    resource_nodes: Vec<ResourceNode>,
    virtual_resources: Vec<VirtualResource>,
    blackboard: FrameGraphBlackboard,
    string_table: HashMap<String, u32>,
    next_string_id: u32,
    merge: bool,
    compiled: bool,
    culled_pass_count: usize,
    device_passes: Vec<DevicePass>,
    resource_allocator: ResourceAllocator,
}

impl FrameGraph {
    pub fn new() -> Self {
        FrameGraph {
            pass_nodes: Vec::new(),
            resource_nodes: Vec::new(),
            virtual_resources: Vec::new(),
            blackboard: FrameGraphBlackboard::default_board(),
            string_table: HashMap::new(),
            next_string_id: 0,
            merge: true,
            compiled: false,
            culled_pass_count: 0,
            device_passes: Vec::new(),
            resource_allocator: ResourceAllocator::default(),
        }
    }

    pub fn string_to_handle(&mut self, name: &str) -> StringHandle {
        if let Some(&idx) = self.string_table.get(name) {
            return StringHandle::new(idx);
        }
        let idx = self.next_string_id;
        self.next_string_id += 1;
        self.string_table.insert(name.to_string(), idx);
        StringHandle::new(idx)
    }

    pub fn handle_to_string(&self, handle: StringHandle) -> Option<&str> {
        for (name, &idx) in &self.string_table {
            if idx == handle.index {
                return Some(name);
            }
        }
        None
    }

    pub fn add_pass<F>(&mut self, insert_point: PassInsertPoint, name: &str, setup: F) -> u32
    where
        F: FnOnce(&mut PassNode),
    {
        let id = self.pass_nodes.len() as u32;
        let mut node = PassNode::new(insert_point, name, id);
        setup(&mut node);
        self.pass_nodes.push(node);
        self.compiled = false;
        id
    }

    pub fn add_pass_with_builder<F>(
        &mut self,
        insert_point: PassInsertPoint,
        name: &str,
        setup: F,
    ) -> u32
    where
        F: FnOnce(&mut PassNodeBuilder),
    {
        let id = self.pass_nodes.len() as u32;
        let mut node = PassNode::new(insert_point, name, id);
        {
            let mut builder = PassNodeBuilder::new(
                &mut node,
                &mut self.resource_nodes,
                &mut self.virtual_resources,
                &mut self.blackboard,
            );
            setup(&mut builder);
        }
        self.pass_nodes.push(node);
        self.compiled = false;
        id
    }

    pub fn create_texture(&mut self, name: &str) -> Handle {
        let res_id = self.virtual_resources.len() as u32;
        self.virtual_resources
            .push(VirtualResource::new_texture(res_id, name, false));
        let node_id = self.resource_nodes.len() as u32;
        self.resource_nodes.push(ResourceNode::new(node_id, name));
        Handle::new(node_id as u16)
    }

    pub fn create_texture_with_desc(&mut self, name: &str, _desc: TextureDescriptor) -> Handle {
        let res_id = self.virtual_resources.len() as u32;
        self.virtual_resources
            .push(VirtualResource::new_texture(res_id, name, false));
        let node_id = self.resource_nodes.len() as u32;
        self.resource_nodes.push(ResourceNode::new(node_id, name));
        Handle::new(node_id as u16)
    }

    pub fn create_buffer(&mut self, name: &str) -> Handle {
        let res_id = self.virtual_resources.len() as u32;
        self.virtual_resources
            .push(VirtualResource::new_buffer(res_id, name, false));
        let node_id = self.resource_nodes.len() as u32;
        self.resource_nodes.push(ResourceNode::new(node_id, name));
        Handle::new(node_id as u16)
    }

    pub fn import_external_texture(&mut self, name: &str) -> Handle {
        let res_id = self.virtual_resources.len() as u32;
        self.virtual_resources
            .push(VirtualResource::new_texture(res_id, name, true));
        let node_id = self.resource_nodes.len() as u32;
        self.resource_nodes.push(ResourceNode::new(node_id, name));
        Handle::new(node_id as u16)
    }

    pub fn import_external_buffer(&mut self, name: &str) -> Handle {
        let res_id = self.virtual_resources.len() as u32;
        self.virtual_resources
            .push(VirtualResource::new_buffer(res_id, name, true));
        let node_id = self.resource_nodes.len() as u32;
        self.resource_nodes.push(ResourceNode::new(node_id, name));
        Handle::new(node_id as u16)
    }

    pub fn present(&mut self, handle: Handle) {
        self.add_pass(INSERT_POINT_POST_PROCESS, "Present", |node| {
            node.read(handle);
            node.side_effect();
        });
    }

    pub fn present_last_version(&mut self, name: &str) {
        let vr_idx = self.virtual_resources.iter().position(|vr| vr.name == name);
        if let Some(idx) = vr_idx {
            let handle = Handle::new(idx as u16);
            self.present(handle);
        }
    }

    pub fn present_from_blackboard(&mut self, name: &str) {
        let val = self.blackboard.get(&name.to_string());
        if val != u32::MAX {
            let handle = Handle::new(val as u16);
            self.present(handle);
        }
    }

    pub fn compile(&mut self) {
        self.sort();
        self.cull();
        self.compute_resource_lifetime();
        if self.merge {
            self.merge_pass_nodes();
        }
        self.compute_store_action_and_memoryless();
        self.generate_device_passes();
        self.compiled = true;
    }

    fn sort(&mut self) {
        self.pass_nodes.sort_by_key(|p| p.insert_point);
        for (i, p) in self.pass_nodes.iter_mut().enumerate() {
            p.id = i as u32;
        }
    }

    fn cull(&mut self) {
        for pass in &mut self.pass_nodes {
            pass.set_ref_count(0);
        }
        for pass_idx in 0..self.pass_nodes.len() {
            let reads = self.pass_nodes[pass_idx].get_reads().to_vec();
            let has_side = self.pass_nodes[pass_idx].has_side_effect();
            for &read_handle in &reads {
                let node_idx = read_handle.index as usize;
                if node_idx < self.resource_nodes.len() {
                    self.resource_nodes[node_idx].ref_count += 1;
                    self.pass_nodes[pass_idx].increment_ref();
                }
            }
            if has_side {
                self.pass_nodes[pass_idx].increment_ref();
            }
        }

        let mut stack: Vec<u32> = Vec::new();
        for (i, pass) in self.pass_nodes.iter().enumerate() {
            if pass.get_ref_count() == 0 {
                stack.push(i as u32);
            }
        }

        while let Some(pass_idx) = stack.pop() {
            let writes: Vec<Handle> = self.pass_nodes[pass_idx as usize].get_writes().to_vec();
            for write_handle in writes {
                let node_idx = write_handle.index as usize;
                if node_idx < self.resource_nodes.len() {
                    let node_ref_count = self.resource_nodes[node_idx].ref_count;
                    if node_ref_count > 0 {
                        self.resource_nodes[node_idx].ref_count -= 1;
                        if self.resource_nodes[node_idx].ref_count == 0 {
                            for j in 0..self.pass_nodes.len() {
                                if self.pass_nodes[j]
                                    .get_reads()
                                    .iter()
                                    .any(|h| h.index == write_handle.index)
                                {
                                    let other_ref = self.pass_nodes[j].get_ref_count();
                                    if other_ref > 0 {
                                        self.pass_nodes[j].set_ref_count(other_ref - 1);
                                        if self.pass_nodes[j].get_ref_count() == 0 {
                                            stack.push(j as u32);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        self.culled_pass_count = self
            .pass_nodes
            .iter()
            .filter(|p| p.get_ref_count() > 0 || p.has_side_effect())
            .count();
    }

    fn compute_resource_lifetime(&mut self) {
        for (pass_idx, pass) in self.pass_nodes.iter().enumerate() {
            if pass.get_ref_count() == 0 && !pass.has_side_effect() {
                continue;
            }
            let pass_id = pass_idx as u32;
            for &h in pass.get_reads().iter().chain(pass.get_writes().iter()) {
                let node_idx = h.index as usize;
                if node_idx < self.resource_nodes.len() {
                    let rn = &mut self.resource_nodes[node_idx];
                    if pass_id < rn.first_pass {
                        rn.first_pass = pass_id;
                    }
                    if pass_id > rn.last_pass {
                        rn.last_pass = pass_id;
                    }
                }
                if node_idx < self.virtual_resources.len() {
                    self.virtual_resources[node_idx].update_lifetime(pass_id);
                }
            }
            for &h in pass.get_writes() {
                let node_idx = h.index as usize;
                if node_idx < self.resource_nodes.len() {
                    self.resource_nodes[node_idx].writer_pass_id = pass_id;
                }
                if node_idx < self.virtual_resources.len() {
                    self.virtual_resources[node_idx].writer_count += 1;
                }
            }
        }
    }

    fn merge_pass_nodes(&mut self) {}

    fn compute_store_action_and_memoryless(&mut self) {
        for i in 0..self.virtual_resources.len() {
            let vr = &self.virtual_resources[i];
            if vr.is_imported() {
                continue;
            }
            let first = vr.first_use_pass_id;
            let last = vr.last_use_pass_id;
            if first < self.pass_nodes.len() as u32 && last < self.pass_nodes.len() as u32 {
                let never_loaded = first == 0
                    || self.pass_nodes[first as usize]
                        .get_writes()
                        .iter()
                        .any(|h| h.index as usize == i);
                let never_stored = last == self.pass_nodes.len() as u32 - 1;
                self.virtual_resources[i].never_loaded = !never_loaded;
                self.virtual_resources[i].never_stored = never_stored;
            }
        }
    }

    fn generate_device_passes(&mut self) {
        self.device_passes.clear();
        for pass in &self.pass_nodes {
            if pass.get_ref_count() == 0 && !pass.has_side_effect() {
                continue;
            }
            let rt =
                DevicePassResourceTable::from_pass_node(pass.get_reads(), pass.get_writes(), 0);
            let subpass = Subpass::new(0, 0);
            let dp = DevicePass::new(
                vec![subpass],
                Vec::new(),
                0,
                rt,
                *pass.get_viewport(),
                pass.get_scissor().clone(),
                Handle::INVALID,
                Handle::INVALID,
            );
            self.device_passes.push(dp);
        }
    }

    pub fn execute(&self) {
        assert!(self.compiled, "FrameGraph must be compiled before execute");
        for dp in &self.device_passes {
            dp.execute();
        }
    }

    pub fn reset(&mut self) {
        self.pass_nodes.clear();
        self.resource_nodes.clear();
        self.virtual_resources.clear();
        self.blackboard.clear();
        self.device_passes.clear();
        self.compiled = false;
        self.culled_pass_count = 0;
        self.resource_allocator.tick();
    }

    pub fn gc(&mut self) {
        self.resource_allocator.gc();
    }

    pub fn get_blackboard(&mut self) -> &mut FrameGraphBlackboard {
        &mut self.blackboard
    }

    pub fn get_pass_count(&self) -> usize {
        self.pass_nodes.len()
    }

    pub fn get_active_pass_count(&self) -> usize {
        self.culled_pass_count
    }

    pub fn get_resource_count(&self) -> usize {
        self.resource_nodes.len()
    }

    pub fn get_virtual_resource_count(&self) -> usize {
        self.virtual_resources.len()
    }

    pub fn enable_merge(&mut self, enable: bool) {
        self.merge = enable;
    }

    pub fn is_compiled(&self) -> bool {
        self.compiled
    }

    pub fn has_pass(&self, name: &str) -> bool {
        self.pass_nodes.iter().any(|p| p.name == name)
    }

    pub fn get_resource_node(&self, handle: Handle) -> Option<&ResourceNode> {
        self.resource_nodes.get(handle.index as usize)
    }

    pub fn get_virtual_resource(&self, index: u32) -> Option<&VirtualResource> {
        self.virtual_resources.get(index as usize)
    }

    pub fn get_pass_node(&self, index: u32) -> Option<&PassNode> {
        self.pass_nodes.get(index as usize)
    }

    pub fn export_graphviz(&self) -> String {
        let mut out = String::from("digraph FrameGraph {\n");
        out.push_str("  rankdir=LR;\n");
        for pass in &self.pass_nodes {
            out.push_str(&format!("  pass_{} [label=\"{}\"];\n", pass.id, pass.name));
            for &h in pass.get_reads() {
                out.push_str(&format!("  res_{} -> pass_{};\n", h.index, pass.id));
            }
            for &h in pass.get_writes() {
                out.push_str(&format!("  pass_{} -> res_{};\n", pass.id, h.index));
            }
        }
        for rn in &self.resource_nodes {
            out.push_str(&format!(
                "  res_{} [label=\"{} v{}\"];\n",
                rn.id, rn.name, rn.version
            ));
        }
        out.push_str("}\n");
        out
    }
}

impl Default for FrameGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_graph_new() {
        let fg = FrameGraph::new();
        assert_eq!(fg.get_pass_count(), 0);
        assert_eq!(fg.get_resource_count(), 0);
        assert!(!fg.is_compiled());
    }

    #[test]
    fn test_frame_graph_add_pass() {
        let mut fg = FrameGraph::new();
        let id = fg.add_pass(0, "ForwardPass", |_node| {});
        assert_eq!(id, 0);
        assert_eq!(fg.get_pass_count(), 1);
    }

    #[test]
    fn test_frame_graph_create_texture() {
        let mut fg = FrameGraph::new();
        let h = fg.create_texture("depth");
        assert!(h.is_valid());
        assert_eq!(fg.get_resource_count(), 1);
    }

    #[test]
    fn test_frame_graph_compile_execute() {
        let mut fg = FrameGraph::new();
        let depth = fg.create_texture("depth");
        fg.add_pass(0, "ForwardPass", |node| {
            node.write(depth);
        });
        fg.compile();
        assert!(fg.is_compiled());
        fg.execute();
    }

    #[test]
    fn test_frame_graph_reset() {
        let mut fg = FrameGraph::new();
        fg.create_texture("color");
        fg.add_pass(0, "Pass", |_| {});
        fg.compile();
        fg.reset();
        assert_eq!(fg.get_pass_count(), 0);
        assert_eq!(fg.get_resource_count(), 0);
        assert!(!fg.is_compiled());
    }

    #[test]
    fn test_frame_graph_blackboard() {
        let mut fg = FrameGraph::new();
        let bb = fg.get_blackboard();
        bb.put("color".to_string(), 1);
        let v = fg.get_blackboard().get(&"color".to_string());
        assert_eq!(v, 1);
    }

    #[test]
    fn test_frame_graph_string_to_handle() {
        let mut fg = FrameGraph::new();
        let h1 = fg.string_to_handle("depth");
        let h2 = fg.string_to_handle("depth");
        let h3 = fg.string_to_handle("color");
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_virtual_resource_lifetime() {
        let mut vr = VirtualResource::new_texture(0, "color", false);
        assert_eq!(vr.first_use_pass_id, u32::MAX);
        vr.update_lifetime(2);
        vr.update_lifetime(5);
        vr.update_lifetime(3);
        assert_eq!(vr.first_use_pass_id, 2);
        assert_eq!(vr.last_use_pass_id, 5);
        assert_eq!(vr.ref_count, 3);
    }

    #[test]
    fn test_frame_graph_culling() {
        let mut fg = FrameGraph::new();
        let color = fg.create_texture("color");
        let unused = fg.create_texture("unused");
        fg.add_pass(0, "ForwardPass", |node| {
            node.write(color);
        });
        fg.add_pass(0, "UnusedPass", |_node| {});
        fg.compile();
        assert!(fg.get_active_pass_count() <= fg.get_pass_count());
    }

    #[test]
    fn test_frame_graph_present() {
        let mut fg = FrameGraph::new();
        let color = fg.create_texture("output");
        fg.add_pass(0, "RenderPass", |node| {
            node.write(color);
        });
        fg.present(color);
        fg.compile();
        assert!(fg.is_compiled());
    }

    #[test]
    fn test_frame_graph_create_buffer() {
        let mut fg = FrameGraph::new();
        let h = fg.create_buffer("uniform");
        assert!(h.is_valid());
        assert_eq!(fg.get_virtual_resource_count(), 1);
    }

    #[test]
    fn test_frame_graph_import_external_buffer() {
        let mut fg = FrameGraph::new();
        let h = fg.import_external_buffer("vb");
        assert!(h.is_valid());
        let vr = fg.get_virtual_resource(h.index as u32);
        assert!(vr.is_some());
        assert!(vr.unwrap().is_imported());
    }

    #[test]
    fn test_virtual_resource_new_version() {
        let mut vr = VirtualResource::new_texture(0, "color", false);
        assert_eq!(vr.version, 0);
        let v1 = vr.new_version();
        assert_eq!(v1, 1);
        let v2 = vr.new_version();
        assert_eq!(v2, 2);
    }

    #[test]
    fn test_resource_node_writer() {
        let rn = ResourceNode::new(0, "color");
        assert_eq!(rn.writer_pass_id, u32::MAX);
        assert_eq!(rn.reader_count, 0);
    }

    #[test]
    fn test_frame_graph_export_graphviz() {
        let mut fg = FrameGraph::new();
        let color = fg.create_texture("color");
        fg.add_pass(0, "ForwardPass", |node| {
            node.write(color);
        });
        fg.compile();
        let dot = fg.export_graphviz();
        assert!(dot.contains("digraph"));
        assert!(dot.contains("ForwardPass"));
    }

    #[test]
    fn test_frame_graph_has_pass() {
        let mut fg = FrameGraph::new();
        fg.add_pass(0, "ShadowPass", |_| {});
        assert!(fg.has_pass("ShadowPass"));
        assert!(!fg.has_pass("UnknownPass"));
    }

    #[test]
    fn test_frame_graph_handle_to_string() {
        let mut fg = FrameGraph::new();
        let h = fg.string_to_handle("depth");
        let name = fg.handle_to_string(h);
        assert_eq!(name, Some("depth"));
    }
}
