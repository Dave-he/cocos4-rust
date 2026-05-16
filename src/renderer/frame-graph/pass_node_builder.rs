/****************************************************************************
Rust port of Cocos Creator PassNodeBuilder
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::blackboard::FrameGraphBlackboard;
use super::pass::{
    Handle, PassBarrierPair, PassNode, RenderTargetAttachment, RenderTargetAttachmentDesc,
};
use super::ResourceNode;
use super::VirtualResource;
use super::VirtualResourceKind;
use crate::renderer::gfx_base::Viewport;

pub struct PassNodeBuilder<'a> {
    pass_node: &'a mut PassNode,
    resource_nodes: &'a mut Vec<ResourceNode>,
    virtual_resources: &'a mut Vec<VirtualResource>,
    blackboard: &'a mut FrameGraphBlackboard,
}

impl<'a> PassNodeBuilder<'a> {
    pub fn new(
        pass_node: &'a mut PassNode,
        resource_nodes: &'a mut Vec<ResourceNode>,
        virtual_resources: &'a mut Vec<VirtualResource>,
        blackboard: &'a mut FrameGraphBlackboard,
    ) -> Self {
        PassNodeBuilder {
            pass_node,
            resource_nodes,
            virtual_resources,
            blackboard,
        }
    }

    pub fn create_texture(&mut self, name: &str) -> Handle {
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

    pub fn import_external(&mut self, name: &str, kind: VirtualResourceKind) -> Handle {
        let res_id = self.virtual_resources.len() as u32;
        let vr = match kind {
            VirtualResourceKind::Texture => VirtualResource::new_texture(res_id, name, true),
            VirtualResourceKind::Buffer => VirtualResource::new_buffer(res_id, name, true),
        };
        self.virtual_resources.push(vr);
        let node_id = self.resource_nodes.len() as u32;
        self.resource_nodes.push(ResourceNode::new(node_id, name));
        Handle::new(node_id as u16)
    }

    pub fn read(&mut self, handle: Handle) -> Handle {
        self.pass_node.read(handle)
    }

    pub fn write(&mut self, handle: Handle) -> Handle {
        self.pass_node.write(handle)
    }

    pub fn write_attachment(
        &mut self,
        handle: Handle,
        level: u8,
        face_id: u8,
        array_position: u8,
        desc: RenderTargetAttachmentDesc,
    ) {
        let attachment = RenderTargetAttachment {
            texture_handle: handle,
            desc,
            level,
            layer: array_position,
            index: face_id,
            store_op: crate::renderer::gfx_base::StoreOp::Discard,
        };
        self.pass_node.create_render_target_attachment(attachment);
        self.pass_node.write(handle);
    }

    pub fn write_attachment_simple(&mut self, handle: Handle, desc: RenderTargetAttachmentDesc) {
        let attachment = RenderTargetAttachment {
            texture_handle: handle,
            desc,
            level: 0,
            layer: 0,
            index: 0,
            store_op: crate::renderer::gfx_base::StoreOp::Discard,
        };
        self.pass_node.create_render_target_attachment(attachment);
        self.pass_node.write(handle);
    }

    pub fn side_effect(&mut self) {
        self.pass_node.side_effect();
    }

    pub fn subpass(&mut self, end: bool, clear_action_ignorable: bool) {
        self.pass_node.set_subpass(end, clear_action_ignorable);
    }

    pub fn set_viewport(&mut self, viewport: Viewport, scissor: crate::renderer::gfx_base::Rect) {
        self.pass_node.set_viewport(viewport, scissor);
    }

    pub fn set_barrier(&mut self, barrier: PassBarrierPair) {
        self.pass_node.set_barrier(barrier);
    }

    pub fn write_to_blackboard(&mut self, name: String, handle: Handle) {
        self.blackboard.put(name, handle.index as u32);
    }

    pub fn read_from_blackboard(&mut self, name: &str) -> Handle {
        let val = self.blackboard.get(&name.to_string());
        if val == u32::MAX {
            Handle::INVALID
        } else {
            Handle::new(val as u16)
        }
    }

    pub fn get_pass_node(&self) -> &PassNode {
        self.pass_node
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pass_node_builder_create_texture() {
        let mut pass_node = PassNode::new(0, "TestPass", 0);
        let mut resource_nodes = Vec::new();
        let mut virtual_resources = Vec::new();
        let mut blackboard = FrameGraphBlackboard::default_board();
        let mut builder = PassNodeBuilder::new(
            &mut pass_node,
            &mut resource_nodes,
            &mut virtual_resources,
            &mut blackboard,
        );
        let h = builder.create_texture("color");
        assert!(h.is_valid());
        assert_eq!(resource_nodes.len(), 1);
        assert_eq!(virtual_resources.len(), 1);
    }

    #[test]
    fn test_pass_node_builder_read_write() {
        let mut pass_node = PassNode::new(0, "TestPass", 0);
        let mut resource_nodes = Vec::new();
        let mut virtual_resources = Vec::new();
        let mut blackboard = FrameGraphBlackboard::default_board();
        let mut builder = PassNodeBuilder::new(
            &mut pass_node,
            &mut resource_nodes,
            &mut virtual_resources,
            &mut blackboard,
        );
        let h = builder.create_texture("depth");
        builder.read(h);
        builder.write(h);
        assert_eq!(pass_node.get_reads().len(), 1);
        assert_eq!(pass_node.get_writes().len(), 1);
    }

    #[test]
    fn test_pass_node_builder_blackboard() {
        let mut pass_node = PassNode::new(0, "TestPass", 0);
        let mut resource_nodes = Vec::new();
        let mut virtual_resources = Vec::new();
        let mut blackboard = FrameGraphBlackboard::default_board();
        let mut builder = PassNodeBuilder::new(
            &mut pass_node,
            &mut resource_nodes,
            &mut virtual_resources,
            &mut blackboard,
        );
        let h = builder.create_texture("output");
        builder.write_to_blackboard("output_color".to_string(), h);
        let read_h = builder.read_from_blackboard("output_color");
        assert!(read_h.is_valid());
        assert_eq!(read_h.index, h.index);
    }

    #[test]
    fn test_pass_node_builder_side_effect() {
        let mut pass_node = PassNode::new(0, "TestPass", 0);
        let mut resource_nodes = Vec::new();
        let mut virtual_resources = Vec::new();
        let mut blackboard = FrameGraphBlackboard::default_board();
        let mut builder = PassNodeBuilder::new(
            &mut pass_node,
            &mut resource_nodes,
            &mut virtual_resources,
            &mut blackboard,
        );
        builder.side_effect();
        assert!(pass_node.has_side_effect());
    }

    #[test]
    fn test_pass_node_builder_import_external() {
        let mut pass_node = PassNode::new(0, "TestPass", 0);
        let mut resource_nodes = Vec::new();
        let mut virtual_resources = Vec::new();
        let mut blackboard = FrameGraphBlackboard::default_board();
        let mut builder = PassNodeBuilder::new(
            &mut pass_node,
            &mut resource_nodes,
            &mut virtual_resources,
            &mut blackboard,
        );
        let h = builder.import_external("backbuffer", VirtualResourceKind::Texture);
        assert!(h.is_valid());
        assert_eq!(virtual_resources.len(), 1);
        assert!(virtual_resources[0].external);
    }
}
