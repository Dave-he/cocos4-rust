/****************************************************************************
Rust port of Cocos Creator 2D Label Component
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::math::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalTextAlignment {
    Left = 0,
    Center = 1,
    Right = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalTextAlignment {
    Top = 0,
    Center = 1,
    Bottom = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overflow {
    None = 0,
    Clamp = 1,
    Shrink = 2,
    ResizeHeight = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheMode {
    None = 0,
    Bitmap = 1,
    Char = 2,
}

#[derive(Debug, Clone)]
pub struct LabelOutline {
    pub enabled: bool,
    pub color: Color,
    pub width: f32,
}

impl Default for LabelOutline {
    fn default() -> Self {
        LabelOutline {
            enabled: false,
            color: Color::BLACK,
            width: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LabelShadow {
    pub enabled: bool,
    pub color: Color,
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
}

impl Default for LabelShadow {
    fn default() -> Self {
        LabelShadow {
            enabled: false,
            color: Color::new(0, 0, 0, 128),
            offset_x: 2.0,
            offset_y: -2.0,
            blur: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Label {
    pub string: String,
    pub horizontal_align: HorizontalTextAlignment,
    pub vertical_align: VerticalTextAlignment,
    pub font_size: f32,
    pub font_family: String,
    pub line_height: f32,
    pub spacing_x: f32,
    pub overflow: Overflow,
    pub enable_wrap: bool,
    pub cache_mode: CacheMode,
    pub color: Color,
    pub outline: LabelOutline,
    pub shadow: LabelShadow,
    pub enabled: bool,
    pub actual_font_size: f32,
    pub content_width: f32,
    pub content_height: f32,
    pub is_bold: bool,
    pub is_italic: bool,
    pub is_underline: bool,
    pub underline_height: f32,
}

impl Label {
    pub fn new() -> Self {
        Label {
            string: String::new(),
            horizontal_align: HorizontalTextAlignment::Left,
            vertical_align: VerticalTextAlignment::Top,
            font_size: 40.0,
            font_family: "Arial".to_string(),
            line_height: 40.0,
            spacing_x: 0.0,
            overflow: Overflow::None,
            enable_wrap: true,
            cache_mode: CacheMode::None,
            color: Color::WHITE,
            outline: LabelOutline::default(),
            shadow: LabelShadow::default(),
            enabled: true,
            actual_font_size: 40.0,
            content_width: 0.0,
            content_height: 0.0,
            is_bold: false,
            is_italic: false,
            is_underline: false,
            underline_height: 1.0,
        }
    }

    pub fn get_string(&self) -> &str {
        &self.string
    }

    pub fn set_string(&mut self, s: &str) {
        self.string = s.to_string();
    }

    pub fn get_font_size(&self) -> f32 {
        self.font_size
    }

    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size;
        self.actual_font_size = size;
    }

    pub fn get_line_height(&self) -> f32 {
        self.line_height
    }

    pub fn set_line_height(&mut self, height: f32) {
        self.line_height = height;
    }

    pub fn get_horizontal_align(&self) -> HorizontalTextAlignment {
        self.horizontal_align
    }

    pub fn set_horizontal_align(&mut self, align: HorizontalTextAlignment) {
        self.horizontal_align = align;
    }

    pub fn get_vertical_align(&self) -> VerticalTextAlignment {
        self.vertical_align
    }

    pub fn set_vertical_align(&mut self, align: VerticalTextAlignment) {
        self.vertical_align = align;
    }

    pub fn get_overflow(&self) -> Overflow {
        self.overflow
    }

    pub fn set_overflow(&mut self, overflow: Overflow) {
        self.overflow = overflow;
    }

    pub fn get_color(&self) -> Color {
        self.color
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn get_cache_mode(&self) -> CacheMode {
        self.cache_mode
    }

    pub fn set_cache_mode(&mut self, mode: CacheMode) {
        self.cache_mode = mode;
    }

    pub fn set_bold(&mut self, bold: bool) {
        self.is_bold = bold;
    }

    pub fn set_italic(&mut self, italic: bool) {
        self.is_italic = italic;
    }

    pub fn set_underline(&mut self, underline: bool) {
        self.is_underline = underline;
    }

    pub fn enable_outline(&mut self, color: Color, width: f32) {
        self.outline.enabled = true;
        self.outline.color = color;
        self.outline.width = width;
    }

    pub fn disable_outline(&mut self) {
        self.outline.enabled = false;
    }

    pub fn enable_shadow(&mut self, color: Color, offset_x: f32, offset_y: f32, blur: f32) {
        self.shadow.enabled = true;
        self.shadow.color = color;
        self.shadow.offset_x = offset_x;
        self.shadow.offset_y = offset_y;
        self.shadow.blur = blur;
    }

    pub fn disable_shadow(&mut self) {
        self.shadow.enabled = false;
    }

    pub fn get_content_width(&self) -> f32 {
        self.content_width
    }

    pub fn get_content_height(&self) -> f32 {
        self.content_height
    }

    pub fn set_content_size(&mut self, width: f32, height: f32) {
        self.content_width = width;
        self.content_height = height;
    }
}

impl Default for Label {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_new() {
        let label = Label::new();
        assert_eq!(label.string, "");
        assert_eq!(label.font_size, 40.0);
        assert_eq!(label.line_height, 40.0);
        assert!(label.enabled);
        assert_eq!(label.color, Color::WHITE);
    }

    #[test]
    fn test_label_set_string() {
        let mut label = Label::new();
        label.set_string("Hello World");
        assert_eq!(label.get_string(), "Hello World");
    }

    #[test]
    fn test_label_set_font_size() {
        let mut label = Label::new();
        label.set_font_size(24.0);
        assert_eq!(label.get_font_size(), 24.0);
        assert_eq!(label.actual_font_size, 24.0);
    }

    #[test]
    fn test_label_alignment() {
        let mut label = Label::new();
        label.set_horizontal_align(HorizontalTextAlignment::Center);
        label.set_vertical_align(VerticalTextAlignment::Center);
        assert_eq!(label.get_horizontal_align(), HorizontalTextAlignment::Center);
        assert_eq!(label.get_vertical_align(), VerticalTextAlignment::Center);
    }

    #[test]
    fn test_label_overflow() {
        let mut label = Label::new();
        label.set_overflow(Overflow::Shrink);
        assert_eq!(label.get_overflow(), Overflow::Shrink);
    }

    #[test]
    fn test_label_color() {
        let mut label = Label::new();
        label.set_color(Color::RED);
        assert_eq!(label.get_color(), Color::RED);
    }

    #[test]
    fn test_label_outline() {
        let mut label = Label::new();
        assert!(!label.outline.enabled);
        label.enable_outline(Color::BLACK, 2.0);
        assert!(label.outline.enabled);
        assert_eq!(label.outline.width, 2.0);
        label.disable_outline();
        assert!(!label.outline.enabled);
    }

    #[test]
    fn test_label_shadow() {
        let mut label = Label::new();
        assert!(!label.shadow.enabled);
        label.enable_shadow(Color::BLACK, 2.0, -2.0, 1.0);
        assert!(label.shadow.enabled);
        assert_eq!(label.shadow.offset_x, 2.0);
        label.disable_shadow();
        assert!(!label.shadow.enabled);
    }

    #[test]
    fn test_label_bold_italic_underline() {
        let mut label = Label::new();
        assert!(!label.is_bold);
        assert!(!label.is_italic);
        assert!(!label.is_underline);
        label.set_bold(true);
        label.set_italic(true);
        label.set_underline(true);
        assert!(label.is_bold);
        assert!(label.is_italic);
        assert!(label.is_underline);
    }
}
