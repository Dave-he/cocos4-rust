/****************************************************************************
Rust port of Cocos Creator 2D Sprite Component
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::math::{Color, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteType {
    Simple = 0,
    Sliced = 1,
    Tiled = 2,
    Filled = 3,
    Mesh = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillType {
    Horizontal = 0,
    Vertical = 1,
    Radial = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeMode {
    Custom = 0,
    Trimmed = 1,
    Raw = 2,
}

#[derive(Debug, Clone)]
pub struct SpriteFrame {
    pub name: String,
    pub width: f32,
    pub height: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub rotated: bool,
    pub uv: [f32; 8],
    pub original_size: Vec2,
    pub rect_x: f32,
    pub rect_y: f32,
    pub rect_w: f32,
    pub rect_h: f32,
}

impl SpriteFrame {
    pub fn new(name: &str, width: f32, height: f32) -> Self {
        SpriteFrame {
            name: name.to_string(),
            width,
            height,
            offset_x: 0.0,
            offset_y: 0.0,
            rotated: false,
            uv: [0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0],
            original_size: Vec2::new(width, height),
            rect_x: 0.0,
            rect_y: 0.0,
            rect_w: width,
            rect_h: height,
        }
    }

    pub fn get_size(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }

    pub fn get_original_size(&self) -> Vec2 {
        self.original_size
    }

    pub fn is_rotated(&self) -> bool {
        self.rotated
    }
}

impl Default for SpriteFrame {
    fn default() -> Self {
        Self::new("", 0.0, 0.0)
    }
}

#[derive(Debug, Clone)]
pub struct Sprite {
    pub sprite_frame: Option<SpriteFrame>,
    pub sprite_type: SpriteType,
    pub fill_type: FillType,
    pub fill_center: Vec2,
    pub fill_start: f32,
    pub fill_range: f32,
    pub size_mode: SizeMode,
    pub color: Color,
    pub trim: bool,
    pub atlas: String,
    pub enabled: bool,
    pub grayscale: bool,
}

impl Sprite {
    pub fn new() -> Self {
        Sprite {
            sprite_frame: None,
            sprite_type: SpriteType::Simple,
            fill_type: FillType::Horizontal,
            fill_center: Vec2::ZERO,
            fill_start: 0.0,
            fill_range: 1.0,
            size_mode: SizeMode::Trimmed,
            color: Color::WHITE,
            trim: true,
            atlas: String::new(),
            enabled: true,
            grayscale: false,
        }
    }

    pub fn get_sprite_frame(&self) -> Option<&SpriteFrame> {
        self.sprite_frame.as_ref()
    }

    pub fn set_sprite_frame(&mut self, frame: Option<SpriteFrame>) {
        self.sprite_frame = frame;
        if let Some(ref f) = self.sprite_frame {
            if self.size_mode != SizeMode::Custom {
                let _ = f.get_size();
            }
        }
    }

    pub fn get_type(&self) -> SpriteType {
        self.sprite_type
    }

    pub fn set_type(&mut self, sprite_type: SpriteType) {
        self.sprite_type = sprite_type;
    }

    pub fn get_fill_type(&self) -> FillType {
        self.fill_type
    }

    pub fn set_fill_type(&mut self, fill_type: FillType) {
        self.fill_type = fill_type;
    }

    pub fn get_fill_center(&self) -> Vec2 {
        self.fill_center
    }

    pub fn set_fill_center(&mut self, center: Vec2) {
        self.fill_center = center;
    }

    pub fn get_fill_start(&self) -> f32 {
        self.fill_start
    }

    pub fn set_fill_start(&mut self, start: f32) {
        self.fill_start = start.clamp(0.0, 1.0);
    }

    pub fn get_fill_range(&self) -> f32 {
        self.fill_range
    }

    pub fn set_fill_range(&mut self, range: f32) {
        self.fill_range = range.clamp(-1.0, 1.0);
    }

    pub fn get_size_mode(&self) -> SizeMode {
        self.size_mode
    }

    pub fn set_size_mode(&mut self, mode: SizeMode) {
        self.size_mode = mode;
    }

    pub fn get_color(&self) -> Color {
        self.color
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn is_grayscale(&self) -> bool {
        self.grayscale
    }

    pub fn set_grayscale(&mut self, grayscale: bool) {
        self.grayscale = grayscale;
    }

    pub fn is_trim(&self) -> bool {
        self.trim
    }

    pub fn set_trim(&mut self, trim: bool) {
        self.trim = trim;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Default for Sprite {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprite_new() {
        let sprite = Sprite::new();
        assert!(sprite.sprite_frame.is_none());
        assert_eq!(sprite.sprite_type, SpriteType::Simple);
        assert_eq!(sprite.fill_type, FillType::Horizontal);
        assert_eq!(sprite.size_mode, SizeMode::Trimmed);
        assert!(sprite.enabled);
    }

    #[test]
    fn test_sprite_frame_new() {
        let frame = SpriteFrame::new("test", 100.0, 200.0);
        assert_eq!(frame.name, "test");
        assert_eq!(frame.width, 100.0);
        assert_eq!(frame.height, 200.0);
        assert!(!frame.is_rotated());
    }

    #[test]
    fn test_sprite_set_frame() {
        let mut sprite = Sprite::new();
        let frame = SpriteFrame::new("hero", 64.0, 64.0);
        sprite.set_sprite_frame(Some(frame));
        assert!(sprite.get_sprite_frame().is_some());
        assert_eq!(sprite.get_sprite_frame().unwrap().name, "hero");
    }

    #[test]
    fn test_sprite_fill_range_clamp() {
        let mut sprite = Sprite::new();
        sprite.set_fill_range(2.0);
        assert_eq!(sprite.get_fill_range(), 1.0);
        sprite.set_fill_range(-2.0);
        assert_eq!(sprite.get_fill_range(), -1.0);
    }

    #[test]
    fn test_sprite_fill_start_clamp() {
        let mut sprite = Sprite::new();
        sprite.set_fill_start(1.5);
        assert_eq!(sprite.get_fill_start(), 1.0);
        sprite.set_fill_start(-0.5);
        assert_eq!(sprite.get_fill_start(), 0.0);
    }

    #[test]
    fn test_sprite_color() {
        let mut sprite = Sprite::new();
        assert_eq!(sprite.get_color(), Color::WHITE);
        sprite.set_color(Color::RED);
        assert_eq!(sprite.get_color(), Color::RED);
    }
}
