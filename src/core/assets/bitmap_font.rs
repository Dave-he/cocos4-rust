/****************************************************************************
Rust port of Cocos Creator BitmapFont
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/
// SPDX-License-Identifier: MIT

use super::asset::AssetBase;
use crate::math::Vec2;

#[derive(Debug, Clone, Default)]
pub struct FontGlyph {
    pub code: u32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub advance: f32,
    pub uv: Vec2,
    pub uv_size: Vec2,
}

#[derive(Debug, Clone, Default)]
pub struct BitmapFontData {
    pub glyphs: Vec<FontGlyph>,
    pub kerning_pairs: Vec<(u32, u32, f32)>,
    pub font_size: u32,
    pub line_height: f32,
    pub texture_path: String,
}

#[derive(Debug)]
#[allow(clippy::derivable_impls)]
pub struct BitmapFont {
    pub base: AssetBase,
    pub font_data: BitmapFontData,
}

impl BitmapFont {
    pub fn new() -> Self {
        BitmapFont {
            base: AssetBase::default(),
            font_data: BitmapFontData::default(),
        }
    }

    pub fn get_glyph(&self, code: u32) -> Option<&FontGlyph> {
        self.font_data.glyphs.iter().find(|g| g.code == code)
    }

    pub fn get_kerning(&self, prev_code: u32, next_code: u32) -> f32 {
        for (prev, next, offset) in &self.font_data.kerning_pairs {
            if *prev == prev_code && *next == next_code {
                return *offset;
            }
        }
        0.0
    }
}

impl Default for BitmapFont {
    #[allow(clippy::derivable_impls)]
    fn default() -> Self {
        BitmapFont {
            base: AssetBase::default(),
            font_data: BitmapFontData::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitmap_font_new() {
        let font = BitmapFont::new();
        assert!(font.font_data.glyphs.is_empty());
        assert!(font.get_glyph(65).is_none());
    }

    #[test]
    fn test_bitmap_font_glyph_lookup() {
        let mut font = BitmapFont::new();
        font.font_data.glyphs.push(FontGlyph {
            code: 65,
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
            advance: 12.0,
            uv: Vec2::new(0.0, 0.0),
            uv_size: Vec2::new(0.1, 0.1),
        });
        let glyph = font.get_glyph(65).unwrap();
        assert_eq!(glyph.code, 65);
        assert_eq!(glyph.advance, 12.0);
    }

    #[test]
    fn test_bitmap_font_kerning() {
        let mut font = BitmapFont::new();
        font.font_data.kerning_pairs.push((65, 66, -2.0));
        assert_eq!(font.get_kerning(65, 66), -2.0);
        assert_eq!(font.get_kerning(65, 67), 0.0);
    }

    #[test]
    fn test_bitmap_font_default() {
        let font = BitmapFont::default();
        assert_eq!(font.font_data.font_size, 0);
    }
}
