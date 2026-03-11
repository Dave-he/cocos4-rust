/****************************************************************************
Rust port of Cocos Creator Renderer Material System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::pass::{IPass as PassIPass, IPassInfo as PassIPassInfo};
use super::program::IProgramInfo;
use crate::base::RefCounted;

pub struct Material {
    pub name: String,
}

impl Material {
    pub fn new(name: &str) -> Self {
        Material {
            name: name.to_string(),
        }
    }
}

pub type MaterialParent = Option<*const Material>;

pub struct IMaterialInstanceInfo {
    pub parent: MaterialParent,
}

#[derive(Debug, Clone)]
pub struct MacroRecord {
    pub value: String,
    pub offset: i32,
}

#[derive(Debug, Clone)]
pub struct IMacroInfo {
    pub name: String,
    pub value: String,
    pub is_default: bool,
}

pub trait MaterialInstance: RefCounted {
    fn get_parent(&self) -> MaterialParent;

    fn recompile_shaders(&mut self, overrides: &MacroRecord);

    fn override_pipeline_states(&mut self, overrides: &PassIPassInfo);

    fn on_pass_state_change(&mut self, dont_notify: bool);
}

pub trait IPassInfo {
    fn get(&self) -> Option<&PassIPass>;
}
