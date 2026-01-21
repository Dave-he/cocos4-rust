/****************************************************************************
Rust port of Cocos Creator GFX Device
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::buffer::Buffer;
use super::queue::Queue;
use super::sampler::Sampler;
use super::swapchain::Swapchain;
use super::texture::Texture;
use crate::base::RefCounted;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryStatus {
    Low = 0,
    Medium = 1,
    High = 2,
}

pub struct DeviceInfo {
    pub name: String,
}

pub trait Device: RefCounted {
    fn initialize(&self, info: &DeviceInfo) -> bool;
    fn destroy(&mut self);
    fn get_memory_status(&self) -> MemoryStatus;
    fn get_num_draw_calls(&self) -> u32;
    fn get_num_instances(&self) -> u32;
    fn get_num_tris(&self) -> u32;
}
