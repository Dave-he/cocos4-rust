/****************************************************************************
Rust port of Cocos Creator GFX State
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::{BlendState, DepthStencilState, PrimitiveMode, RasterizerState};

#[derive(Debug, Clone)]
pub struct InputState {
    pub attributes: Vec<super::VertexAttribute>,
}

impl Default for InputState {
    fn default() -> Self {
        InputState {
            attributes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PipelineLayoutInfo {
    pub set_layouts: Vec<u32>,
}

impl Default for PipelineLayoutInfo {
    fn default() -> Self {
        PipelineLayoutInfo {
            set_layouts: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct GfxPipelineLayout {
    pub id: u32,
    pub info: PipelineLayoutInfo,
}

impl GfxPipelineLayout {
    pub fn new(id: u32, info: PipelineLayoutInfo) -> Self {
        GfxPipelineLayout { id, info }
    }
}
