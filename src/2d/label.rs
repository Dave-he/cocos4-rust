/****************************************************************************
Rust port of Cocos Creator 2D Label Component
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::RefCounted;
use crate::core::scene_graph::Node;

#[derive(Debug, Clone)]
pub struct Overflow {
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct LabelLayout {
    pub font_size: f32,
    pub line_height: f32,
    pub spacing_x: f32,
    pub spacing_y: f32,
    pub horizontal_align: f32,
    pub vertical_align: f32,
}

#[derive(Debug, Clone)]
pub struct LabelOutline {
    pub width: f32,
    pub height: f32,
    pub color: [u8; 4],
}

pub trait Label: RefCounted {
    fn get_font_size(&self) -> f32;
    fn set_font_size(&mut self, size: f32);
    fn get_line_height(&self) -> f32;
    fn set_line_height(&mut self, height: f32);
    fn get_spacing(&self) -> [f32; 2];
    fn set_spacing(&mut self, x: f32, y: f32);
    fn get_horizontal_align(&self) -> f32;
    fn set_horizontal_align(&mut self, align: f32);
    fn get_vertical_align(&self) -> f32;
    fn set_vertical_align(&mut self, align: f32);
}
