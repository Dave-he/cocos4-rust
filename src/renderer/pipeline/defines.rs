/****************************************************************************
Rust port of Cocos Creator Pipeline Define System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefineType {
    Int = 0,
    Bool = 1,
    String = 2,
    Number = 3,
    Buffer = 4,
}

#[derive(Debug, Clone)]
pub struct MacroRecord {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct MacroValue {
    pub define_type: DefineType,
    pub value: String,
}
