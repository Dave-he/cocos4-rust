use super::asset::AssetBase;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontType {
    TrueType,
    SystemFont,
    Bitmap,
}

impl Default for FontType {
    fn default() -> Self {
        FontType::TrueType
    }
}

#[derive(Debug)]
pub struct Font {
    pub base: AssetBase,
    pub font_type: FontType,
    pub family_name: String,
}

impl Font {
    pub fn new() -> Self {
        Font {
            base: AssetBase::new(),
            font_type: FontType::TrueType,
            family_name: String::new(),
        }
    }

    pub fn get_family_name(&self) -> &str {
        &self.family_name
    }

    pub fn set_family_name(&mut self, name: &str) {
        self.family_name = name.to_string();
    }

    pub fn get_font_type(&self) -> FontType {
        self.font_type
    }
}

impl Default for Font {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct BitmapFont {
    pub base: Font,
    pub sprite_frame_uuid: String,
    pub font_size: f32,
    pub char_map: Vec<BitmapFontChar>,
}

#[derive(Debug, Clone)]
pub struct BitmapFontChar {
    pub code: u32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub x_advance: f32,
}

impl BitmapFont {
    pub fn new() -> Self {
        let mut base = Font::new();
        base.font_type = FontType::Bitmap;
        BitmapFont {
            base,
            sprite_frame_uuid: String::new(),
            font_size: 0.0,
            char_map: Vec::new(),
        }
    }

    pub fn get_char(&self, code: u32) -> Option<&BitmapFontChar> {
        self.char_map.iter().find(|c| c.code == code)
    }
}

impl Default for BitmapFont {
    fn default() -> Self {
        Self::new()
    }
}
