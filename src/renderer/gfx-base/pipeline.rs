/****************************************************************************
Rust port of Cocos Creator GFX Pipeline State
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::{BlendFactor, BlendOp, ComparisonFunc, PrimitiveMode, StencilOp};

#[derive(Debug, Clone)]
pub struct RasterizerState {
    pub is_discard: bool,
    pub polygon_mode: u32,
    pub shade_model: u32,
    pub cull_mode: u32,
    pub is_front_face_ccw: bool,
    pub depth_bias_enabled: bool,
    pub depth_bias: f32,
    pub depth_bias_clamp: f32,
    pub depth_bias_slope: f32,
    pub is_depth_clip: bool,
    pub is_multi_sample: bool,
    pub line_width: f32,
}

impl Default for RasterizerState {
    fn default() -> Self {
        RasterizerState {
            is_discard: false,
            polygon_mode: 0,
            shade_model: 0,
            cull_mode: 2,
            is_front_face_ccw: false,
            depth_bias_enabled: false,
            depth_bias: 0.0,
            depth_bias_clamp: 0.0,
            depth_bias_slope: 0.0,
            is_depth_clip: true,
            is_multi_sample: false,
            line_width: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DepthStencilState {
    pub depth_test: bool,
    pub depth_write: bool,
    pub depth_func: ComparisonFunc,
    pub stencil_test_front: bool,
    pub stencil_func_front: ComparisonFunc,
    pub stencil_read_mask_front: u32,
    pub stencil_write_mask_front: u32,
    pub stencil_fail_op_front: StencilOp,
    pub stencil_z_fail_op_front: StencilOp,
    pub stencil_pass_op_front: StencilOp,
    pub stencil_ref_front: u32,
    pub stencil_test_back: bool,
    pub stencil_func_back: ComparisonFunc,
    pub stencil_read_mask_back: u32,
    pub stencil_write_mask_back: u32,
    pub stencil_fail_op_back: StencilOp,
    pub stencil_z_fail_op_back: StencilOp,
    pub stencil_pass_op_back: StencilOp,
    pub stencil_ref_back: u32,
}

impl Default for DepthStencilState {
    fn default() -> Self {
        DepthStencilState {
            depth_test: true,
            depth_write: true,
            depth_func: ComparisonFunc::Less,
            stencil_test_front: false,
            stencil_func_front: ComparisonFunc::Always,
            stencil_read_mask_front: 0xFFFFFFFF,
            stencil_write_mask_front: 0xFFFFFFFF,
            stencil_fail_op_front: StencilOp::Keep,
            stencil_z_fail_op_front: StencilOp::Keep,
            stencil_pass_op_front: StencilOp::Keep,
            stencil_ref_front: 1,
            stencil_test_back: false,
            stencil_func_back: ComparisonFunc::Always,
            stencil_read_mask_back: 0xFFFFFFFF,
            stencil_write_mask_back: 0xFFFFFFFF,
            stencil_fail_op_back: StencilOp::Keep,
            stencil_z_fail_op_back: StencilOp::Keep,
            stencil_pass_op_back: StencilOp::Keep,
            stencil_ref_back: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlendTarget {
    pub blend: bool,
    pub blend_src: BlendFactor,
    pub blend_dst: BlendFactor,
    pub blend_eq: BlendOp,
    pub blend_src_alpha: BlendFactor,
    pub blend_dst_alpha: BlendFactor,
    pub blend_alpha_eq: BlendOp,
    pub blend_color_mask: u32,
}

impl Default for BlendTarget {
    fn default() -> Self {
        BlendTarget {
            blend: false,
            blend_src: BlendFactor::One,
            blend_dst: BlendFactor::Zero,
            blend_eq: BlendOp::Add,
            blend_src_alpha: BlendFactor::One,
            blend_dst_alpha: BlendFactor::Zero,
            blend_alpha_eq: BlendOp::Add,
            blend_color_mask: 0xF,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlendState {
    pub is_independent: bool,
    pub blend_color: [f32; 4],
    pub targets: Vec<BlendTarget>,
}

impl Default for BlendState {
    fn default() -> Self {
        BlendState {
            is_independent: false,
            blend_color: [0.0, 0.0, 0.0, 0.0],
            targets: vec![BlendTarget::default()],
        }
    }
}

#[derive(Debug, Clone)]
pub struct PipelineStateInfo {
    pub shader_id: u32,
    pub render_pass_id: u32,
    pub primitive: PrimitiveMode,
    pub rasterizer_state: RasterizerState,
    pub depth_stencil_state: DepthStencilState,
    pub blend_state: BlendState,
    pub dynamic_states: u32,
    pub bind_point: u32,
}

impl Default for PipelineStateInfo {
    fn default() -> Self {
        PipelineStateInfo {
            shader_id: 0,
            render_pass_id: 0,
            primitive: PrimitiveMode::TriangleList,
            rasterizer_state: RasterizerState::default(),
            depth_stencil_state: DepthStencilState::default(),
            blend_state: BlendState::default(),
            dynamic_states: 0,
            bind_point: 0,
        }
    }
}

#[derive(Debug)]
pub struct GfxPipelineState {
    pub id: u32,
    pub info: PipelineStateInfo,
}

impl GfxPipelineState {
    pub fn new(id: u32, info: PipelineStateInfo) -> Self {
        GfxPipelineState { id, info }
    }

    pub fn get_primitive_mode(&self) -> PrimitiveMode {
        self.info.primitive
    }

    pub fn is_depth_test_enabled(&self) -> bool {
        self.info.depth_stencil_state.depth_test
    }

    pub fn is_blend_enabled(&self) -> bool {
        self.info.blend_state.targets.iter().any(|t| t.blend)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_state_default() {
        let info = PipelineStateInfo::default();
        assert_eq!(info.primitive, PrimitiveMode::TriangleList);
        assert!(info.depth_stencil_state.depth_test);
    }

    #[test]
    fn test_pipeline_state_new() {
        let pso = GfxPipelineState::new(1, PipelineStateInfo::default());
        assert_eq!(pso.get_primitive_mode(), PrimitiveMode::TriangleList);
        assert!(pso.is_depth_test_enabled());
        assert!(!pso.is_blend_enabled());
    }
}
