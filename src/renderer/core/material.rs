/****************************************************************************
Rust port of Cocos Creator Renderer Material System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::super::pass::IPassInfo;
use super::super::program::IProgramInfo;
use crate::base::RefCounted;

pub type MaterialParent = Option<*const super::super::material::Material>;

pub struct IMaterialInstanceInfo {
    pub parent: MaterialParent,
}

pub struct MacroRecord {
    pub value: String,
    pub offset: i32,
}

pub struct IMacroInfo {
    pub name: String,
    pub value: String,
    pub is_default: bool,
}

pub trait MaterialInstance: RefCounted {
    fn get_parent(&self) -> MaterialParent;

    fn recompile_shaders(&mut self, overrides: &MacroRecord);

    fn override_pipeline_states(&mut self, overrides: &super::super::pass::IPassInfo);

    fn on_pass_state_change(&mut self, dont_notify: bool);
}

pub trait IPassInfo {
    fn get(&self) -> Option<&super::super::pass::IPass>;
}
