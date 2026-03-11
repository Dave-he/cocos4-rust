/****************************************************************************
Rust port of Cocos Creator Renderer Program System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::renderer::gfx_base::device::Device;
use crate::renderer::gfx_base::sampler::Sampler;
use crate::renderer::gfx_base::shader::Shader;
use crate::renderer::gfx_base::texture::Texture;
use crate::base::RefCounted;
use super::material::MacroRecord;

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
