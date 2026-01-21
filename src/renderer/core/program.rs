/****************************************************************************
Rust port of Cocos Creator Renderer Program System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::super::super::gfx_base::device::Device;
use super::super::super::gfx_base::sampler::Sampler;
use super::super::super::gfx_base::shader::Shader;
use super::super::super::gfx_base::texture::Texture;
use crate::base::RefCounted;

#[derive(Debug, Clone)]
pub struct IDefineRecord {
    pub defines: Vec<MacroRecord>,
}

#[derive(Debug, Clone)]
pub struct ITemplateInfo {
    pub name: String,
    pub defines: Vec<MacroRecord>,
}

pub trait IProgramInfo {
    fn get(&self) -> Option<&IProgramInfo>;
}

pub struct ShaderInfo {
    pub name: String,
    pub defines: Vec<MacroRecord>,
}
