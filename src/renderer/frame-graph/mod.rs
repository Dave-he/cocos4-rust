/****************************************************************************
Rust port of Cocos Creator Frame Graph System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

pub mod blackboard;
pub mod pass;

pub use blackboard::*;
pub use pass::*;

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// ResourceNode
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ResourceNode {
    pub id: u32,
    pub name: String,
    pub version: u8,
    pub ref_count: u32,
    pub first_pass: u32,
    pub last_pass: u32,
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
        }
    }
}

// ---------------------------------------------------------------------------
// VirtualResource
// ---------------------------------------------------------------------------

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
    pub writer_pass_id: u32,
    pub first_use_pass_id: u32,
    pub last_use_pass_id: u32,
    pub ref_count: u32,
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
            writer_pass_id: u32::MAX,
            first_use_pass_id: u32::MAX,
            last_use_pass_id: 0,
            ref_count: 0,
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
            writer_pass_id: u32::MAX,
            first_use_pass_id: u32::MAX,
            last_use_pass_id: 0,
            ref_count: 0,
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
}

// ---------------------------------------------------------------------------
// FrameGraph
// ---------------------------------------------------------------------------

pub struct FrameGraph {
    pass_nodes: Vec<PassNode>,
    resource_nodes: Vec<ResourceNode>,
    virtual_resources: Vec<VirtualResource>,
    blackboard: FrameGraphBlackboard,
    string_table: HashMap<String, u32>,
    next_string_id: u32,
    merge: bool,
    compiled: bool,
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

    pub fn create_texture(&mut self, name: &str) -> Handle {
        let res_id = self.virtual_resources.len() as u32;
        self.virtual_resources.push(VirtualResource::new_texture(res_id, name, false));
        let node_id = self.resource_nodes.len() as u32;
        self.resource_nodes.push(ResourceNode::new(node_id, name));
        Handle::new(node_id as u16)
    }

    pub fn import_external_texture(&mut self, name: &str) -> Handle {
        let res_id = self.virtual_resources.len() as u32;
        self.virtual_resources.push(VirtualResource::new_texture(res_id, name, true));
        let node_id = self.resource_nodes.len() as u32;
        self.resource_nodes.push(ResourceNode::new(node_id, name));
        Handle::new(node_id as u16)
    }

    pub fn compile(&mut self) {
        self.cull();
        self.compute_resource_lifetime();
        self.compiled = true;
    }

    fn cull(&mut self) {
        for pass in &self.pass_nodes {
            for &read_handle in pass.get_reads() {
                if let Some(node) = self.resource_nodes.get_mut(read_handle.index as usize) {
                    node.ref_count += 1;
                }
            }
        }
    }

    fn compute_resource_lifetime(&mut self) {
        for (pass_idx, pass) in self.pass_nodes.iter().enumerate() {
            let pass_id = pass_idx as u32;
            for &h in pass.get_reads().iter().chain(pass.get_writes().iter()) {
                let node_id = h.index as usize;
                if let Some(rn) = self.resource_nodes.get_mut(node_id) {
                    if pass_id < rn.first_pass { rn.first_pass = pass_id; }
                    if pass_id > rn.last_pass { rn.last_pass = pass_id; }
                }
                if let Some(vr) = self.virtual_resources.get_mut(node_id) {
                    vr.update_lifetime(pass_id);
                }
            }
        }
    }

    pub fn execute(&self) {
        assert!(self.compiled, "FrameGraph must be compiled before execute");
    }

    pub fn reset(&mut self) {
        self.pass_nodes.clear();
        self.resource_nodes.clear();
        self.virtual_resources.clear();
        self.blackboard.clear();
        self.compiled = false;
    }

    pub fn get_blackboard(&mut self) -> &mut FrameGraphBlackboard {
        &mut self.blackboard
    }

    pub fn get_pass_count(&self) -> usize {
        self.pass_nodes.len()
    }

    pub fn get_resource_count(&self) -> usize {
        self.resource_nodes.len()
    }

    pub fn enable_merge(&mut self, enable: bool) {
        self.merge = enable;
    }

    pub fn is_compiled(&self) -> bool {
        self.compiled
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
        fg.add_pass(0, "ForwardPass", |node| { node.write(depth); });
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
}
