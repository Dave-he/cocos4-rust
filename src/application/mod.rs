/****************************************************************************
Rust port of Cocos Creator Application Manager
Original C++ version Copyright (c) 2017-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::RefCounted;

pub trait BaseApplication: RefCounted {
    fn get_id(&self) -> u64;
    fn run(&mut self);
    fn get_title(&self) -> String;
}

pub trait ApplicationManager: RefCounted {
    fn get_instance(&self) -> Option<Box<dyn BaseApplication>>;
}

pub trait PlatformInterface: RefCounted {
    fn create_window(&mut self, id: u32, title: &str, width: i32, height: i32);
}
