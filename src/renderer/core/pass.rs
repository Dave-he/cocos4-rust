/****************************************************************************
Rust port of Cocos Creator Renderer Pass System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::material::MaterialInstance;
use crate::base::RefCounted;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassType {
    Compute = 0,
    Graphics = 1,
}

#[derive(Debug, Clone)]
pub struct IPassInfo {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct PassOverrides {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct CallbackPass {
    pub name: String,
    pub priority: i32,
}

impl CallbackPass {
    pub fn new(name: &str, priority: i32) -> Self {
        Self {
            name: name.to_string(),
            priority,
        }
    }
}

pub trait IPass: RefCounted {
    fn get(&self) -> Option<&IPassInfo>;
}

pub trait RenderableComponent: RefCounted {}

pub struct PassInstance {
    pub parent: Option<*const MaterialInstance>,
    pub owner: Option<*const dyn RenderableComponent>,
    pub sub_model_index: usize,
}

impl PassInstance {
    pub fn new(parent: Option<*const MaterialInstance>) -> Self {
        PassInstance {
            parent,
            owner: None,
            sub_model_index: 0,
        }
    }
}

impl Drop for PassInstance {
    fn drop(&mut self) {
        self.parent = None;
    }
}
