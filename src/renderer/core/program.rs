/****************************************************************************
Rust port of Cocos Creator Renderer Program System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

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

pub trait IProgramInfoTrait {
    fn get_info(&self) -> Option<&ProgramInfo>;
}

#[derive(Debug, Clone)]
pub struct ProgramInfo {
    pub name: String,
    pub defines: Vec<MacroRecord>,
}

#[derive(Debug, Clone)]
pub struct ShaderInfo {
    pub name: String,
    pub defines: Vec<MacroRecord>,
}

impl ShaderInfo {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            defines: Vec::new(),
        }
    }
}
