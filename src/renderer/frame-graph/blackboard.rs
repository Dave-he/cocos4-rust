/****************************************************************************
Rust port of Cocos Creator Blackboard System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::RefCounted;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HandleIndexType {
    Uninitialized = 0,
    IndexType = 1,
}

#[derive(Debug, Clone)]
pub struct StringHandle {
    pub index_type: HandleIndexType,
}

pub trait Blackboard: RefCounted {
    fn put(&mut self, name: String, value: String);
    fn get(&self, name: &str) -> Option<String>;
    fn clear(&mut self);
    fn has(&self, name: &str) -> bool;
}
