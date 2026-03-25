/****************************************************************************
Rust port of Cocos Creator Frame Graph Pass System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::renderer::gfx_base::{Color, LoadOp, StoreOp, Viewport, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Handle {
    pub index: u16,
}

impl Handle {
    pub const INVALID: Handle = Handle { index: u16::MAX };

    pub fn new(index: u16) -> Self {
        Handle { index }
    }

    pub fn is_valid(&self) -> bool {
        self.index != u16::MAX
    }
}

pub type PassInsertPoint = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderTargetUsage {
    Color,
    Depth,
    Stencil,
    DepthStencil,
}

impl Default for RenderTargetUsage {
    fn default() -> Self {
        RenderTargetUsage::Color
    }
}

#[derive(Debug, Clone)]
pub struct RenderTargetAttachmentDesc {
    pub usage: RenderTargetUsage,
    pub slot: u8,
    pub write_mask: u8,
    pub load_op: LoadOp,
    pub clear_color: Color,
    pub clear_depth: f32,
    pub clear_stencil: u8,
    pub begin_accesses: u64,
    pub end_accesses: u64,
}

impl Default for RenderTargetAttachmentDesc {
    fn default() -> Self {
        RenderTargetAttachmentDesc {
            usage: RenderTargetUsage::Color,
            slot: 0xff,
            write_mask: 0xff,
            load_op: LoadOp::Discard,
            clear_color: Color::default(),
            clear_depth: 1.0,
            clear_stencil: 0,
            begin_accesses: 0,
            end_accesses: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderTargetAttachment {
    pub texture_handle: Handle,
    pub desc: RenderTargetAttachmentDesc,
    pub level: u8,
    pub layer: u8,
    pub index: u8,
    pub store_op: StoreOp,
}

impl Default for RenderTargetAttachment {
    fn default() -> Self {
        RenderTargetAttachment {
            texture_handle: Handle::INVALID,
            desc: RenderTargetAttachmentDesc::default(),
            level: 0,
            layer: 0,
            index: 0,
            store_op: StoreOp::Discard,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Unknown,
    Buffer,
    Texture,
}

impl Default for ResourceType {
    fn default() -> Self {
        ResourceType::Unknown
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarrierType {
    Full,
    Split,
    SplitBegin,
    SplitEnd,
}

impl Default for BarrierType {
    fn default() -> Self {
        BarrierType::Full
    }
}

#[derive(Debug, Clone, Default)]
pub struct AccessStatus {
    pub pass_type: u32,
    pub visibility: u32,
    pub access: u32,
}

#[derive(Debug, Clone, Default)]
pub struct Range {
    pub base: usize,
    pub len: usize,
}

#[derive(Debug, Clone)]
pub struct ResourceBarrier {
    pub resource_type: ResourceType,
    pub barrier_type: BarrierType,
    pub handle: Handle,
    pub begin_status: AccessStatus,
    pub end_status: AccessStatus,
    pub layer_range: Range,
    pub mip_or_buffer_range: Range,
}

impl Default for ResourceBarrier {
    fn default() -> Self {
        ResourceBarrier {
            resource_type: ResourceType::Unknown,
            barrier_type: BarrierType::Full,
            handle: Handle::INVALID,
            begin_status: AccessStatus::default(),
            end_status: AccessStatus::default(),
            layer_range: Range::default(),
            mip_or_buffer_range: Range::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PassBarrierPair {
    pub front_barriers: Vec<ResourceBarrier>,
    pub rear_barriers: Vec<ResourceBarrier>,
}

#[derive(Debug)]
pub struct PassNode {
    pub name: String,
    pub id: u32,
    pub insert_point: PassInsertPoint,
    reads: Vec<Handle>,
    writes: Vec<Handle>,
    attachments: Vec<RenderTargetAttachment>,
    ref_count: u32,
    side_effect: bool,
    subpass: bool,
    subpass_end: bool,
    clear_action_ignorable: bool,
    custom_viewport: bool,
    viewport: Viewport,
    scissor: Rect,
    barriers: PassBarrierPair,
}

impl PassNode {
    pub fn new(insert_point: PassInsertPoint, name: &str, id: u32) -> Self {
        PassNode {
            name: name.to_string(),
            id,
            insert_point,
            reads: Vec::new(),
            writes: Vec::new(),
            attachments: Vec::new(),
            ref_count: 0,
            side_effect: false,
            subpass: false,
            subpass_end: false,
            clear_action_ignorable: false,
            custom_viewport: false,
            viewport: Viewport::default(),
            scissor: Rect::default(),
            barriers: PassBarrierPair::default(),
        }
    }

    pub fn read(&mut self, handle: Handle) -> Handle {
        self.reads.push(handle);
        handle
    }

    pub fn write(&mut self, handle: Handle) -> Handle {
        self.writes.push(handle);
        handle
    }

    pub fn create_render_target_attachment(&mut self, attachment: RenderTargetAttachment) {
        self.attachments.push(attachment);
    }

    pub fn set_barrier(&mut self, barrier: PassBarrierPair) {
        self.barriers = barrier;
    }

    pub fn side_effect(&mut self) {
        self.side_effect = true;
    }

    pub fn set_subpass(&mut self, end: bool, clear_action_ignorable: bool) {
        self.subpass = true;
        self.subpass_end = end;
        self.clear_action_ignorable = clear_action_ignorable;
    }

    pub fn set_viewport(&mut self, viewport: Viewport, scissor: Rect) {
        self.custom_viewport = true;
        self.viewport = viewport;
        self.scissor = scissor;
    }

    pub fn get_barriers(&self) -> &PassBarrierPair {
        &self.barriers
    }

    pub fn get_reads(&self) -> &[Handle] {
        &self.reads
    }

    pub fn get_writes(&self) -> &[Handle] {
        &self.writes
    }

    pub fn get_attachments(&self) -> &[RenderTargetAttachment] {
        &self.attachments
    }

    pub fn has_side_effect(&self) -> bool {
        self.side_effect
    }

    pub fn is_subpass(&self) -> bool {
        self.subpass
    }

    pub fn has_custom_viewport(&self) -> bool {
        self.custom_viewport
    }

    pub fn get_viewport(&self) -> &Viewport {
        &self.viewport
    }

    pub fn get_scissor(&self) -> &Rect {
        &self.scissor
    }

    pub fn get_ref_count(&self) -> u32 {
        self.ref_count
    }

    pub fn increment_ref(&mut self) {
        self.ref_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_valid() {
        let h = Handle::new(0);
        assert!(h.is_valid());
        assert!(!Handle::INVALID.is_valid());
    }

    #[test]
    fn test_pass_node_new() {
        let node = PassNode::new(0, "ForwardPass", 1);
        assert_eq!(node.name, "ForwardPass");
        assert_eq!(node.id, 1);
        assert!(node.get_reads().is_empty());
        assert!(node.get_writes().is_empty());
        assert!(!node.has_side_effect());
    }

    #[test]
    fn test_pass_node_read_write() {
        let mut node = PassNode::new(0, "TestPass", 0);
        let h = Handle::new(1);
        node.read(h);
        node.write(h);
        assert_eq!(node.get_reads().len(), 1);
        assert_eq!(node.get_writes().len(), 1);
    }

    #[test]
    fn test_pass_node_side_effect() {
        let mut node = PassNode::new(0, "TestPass", 0);
        assert!(!node.has_side_effect());
        node.side_effect();
        assert!(node.has_side_effect());
    }

    #[test]
    fn test_pass_node_viewport() {
        let mut node = PassNode::new(0, "TestPass", 0);
        assert!(!node.has_custom_viewport());
        node.set_viewport(Viewport::default(), Rect::default());
        assert!(node.has_custom_viewport());
    }

    #[test]
    fn test_render_target_attachment_default() {
        let a = RenderTargetAttachment::default();
        assert!(!a.texture_handle.is_valid());
        assert_eq!(a.clear_depth_value(), 1.0);
    }
}

impl RenderTargetAttachment {
    pub fn clear_depth_value(&self) -> f32 {
        self.desc.clear_depth
    }
}
