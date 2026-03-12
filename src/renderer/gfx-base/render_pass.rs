/****************************************************************************
Rust port of Cocos Creator GFX Render Pass
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::{Format, LoadOp, SampleCount, StoreOp};

#[derive(Debug, Clone)]
pub struct ColorAttachment {
    pub format: Format,
    pub sample_count: SampleCount,
    pub load_op: LoadOp,
    pub store_op: StoreOp,
    pub stencil_load_op: LoadOp,
    pub stencil_store_op: StoreOp,
}

impl Default for ColorAttachment {
    fn default() -> Self {
        ColorAttachment {
            format: Format::RGBA8,
            sample_count: SampleCount::X1,
            load_op: LoadOp::Clear,
            store_op: StoreOp::Store,
            stencil_load_op: LoadOp::Discard,
            stencil_store_op: StoreOp::Discard,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DepthStencilAttachment {
    pub format: Format,
    pub sample_count: SampleCount,
    pub depth_load_op: LoadOp,
    pub depth_store_op: StoreOp,
    pub stencil_load_op: LoadOp,
    pub stencil_store_op: StoreOp,
    pub is_depth_stencil: bool,
}

impl Default for DepthStencilAttachment {
    fn default() -> Self {
        DepthStencilAttachment {
            format: Format::D24S8,
            sample_count: SampleCount::X1,
            depth_load_op: LoadOp::Clear,
            depth_store_op: StoreOp::Discard,
            stencil_load_op: LoadOp::Discard,
            stencil_store_op: StoreOp::Discard,
            is_depth_stencil: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SubpassInfo {
    pub inputs: Vec<u32>,
    pub colors: Vec<u32>,
    pub resolves: Vec<u32>,
    pub preserves: Vec<u32>,
    pub depth_stencil: i32,
    pub depth_stencil_resolve: i32,
    pub depth_resolve_mode: u32,
    pub stencil_resolve_mode: u32,
}

impl Default for SubpassInfo {
    fn default() -> Self {
        SubpassInfo {
            inputs: Vec::new(),
            colors: Vec::new(),
            resolves: Vec::new(),
            preserves: Vec::new(),
            depth_stencil: -1,
            depth_stencil_resolve: -1,
            depth_resolve_mode: 0,
            stencil_resolve_mode: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderPassInfo {
    pub color_attachments: Vec<ColorAttachment>,
    pub depth_stencil_attachment: DepthStencilAttachment,
    pub subpasses: Vec<SubpassInfo>,
}

impl Default for RenderPassInfo {
    fn default() -> Self {
        RenderPassInfo {
            color_attachments: vec![ColorAttachment::default()],
            depth_stencil_attachment: DepthStencilAttachment::default(),
            subpasses: vec![SubpassInfo::default()],
        }
    }
}

#[derive(Debug)]
pub struct GfxRenderPass {
    pub id: u32,
    pub info: RenderPassInfo,
}

impl GfxRenderPass {
    pub fn new(id: u32, info: RenderPassInfo) -> Self {
        GfxRenderPass { id, info }
    }

    pub fn get_color_attachment_count(&self) -> usize {
        self.info.color_attachments.len()
    }

    pub fn has_depth_stencil(&self) -> bool {
        self.info.depth_stencil_attachment.is_depth_stencil
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_pass_new() {
        let info = RenderPassInfo::default();
        let rp = GfxRenderPass::new(1, info);
        assert_eq!(rp.get_color_attachment_count(), 1);
        assert!(rp.has_depth_stencil());
    }

    #[test]
    fn test_color_attachment_default() {
        let ca = ColorAttachment::default();
        assert_eq!(ca.format, Format::RGBA8);
        assert_eq!(ca.load_op, LoadOp::Clear);
        assert_eq!(ca.store_op, StoreOp::Store);
    }
}
