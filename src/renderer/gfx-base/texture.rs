/****************************************************************************
Rust port of Cocos Creator GFX Texture
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::{Format, SampleCount, TextureFlags, TextureType, TextureUsage};

#[derive(Debug, Clone)]
pub struct TextureInfo {
    pub tex_type: TextureType,
    pub usage: TextureUsage,
    pub format: Format,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub array_layers: u32,
    pub mip_levels: u32,
    pub samples: SampleCount,
    pub flags: TextureFlags,
    pub layer_count: u32,
}

impl Default for TextureInfo {
    fn default() -> Self {
        TextureInfo {
            tex_type: TextureType::Tex2D,
            usage: TextureUsage::SAMPLED,
            format: Format::RGBA8,
            width: 1,
            height: 1,
            depth: 1,
            array_layers: 1,
            mip_levels: 1,
            samples: SampleCount::X1,
            flags: TextureFlags::NONE,
            layer_count: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextureViewInfo {
    pub texture_id: u32,
    pub tex_type: TextureType,
    pub format: Format,
    pub base_level: u32,
    pub level_count: u32,
    pub base_layer: u32,
    pub layer_count: u32,
}

impl Default for TextureViewInfo {
    fn default() -> Self {
        TextureViewInfo {
            texture_id: 0,
            tex_type: TextureType::Tex2D,
            format: Format::RGBA8,
            base_level: 0,
            level_count: 1,
            base_layer: 0,
            layer_count: 1,
        }
    }
}

#[derive(Debug)]
pub struct GfxTexture {
    pub id: u32,
    pub info: TextureInfo,
    pub data: Vec<u8>,
}

impl GfxTexture {
    pub fn new(id: u32, info: TextureInfo) -> Self {
        let size = Self::calc_size(&info);
        GfxTexture {
            id,
            info,
            data: vec![0u8; size],
        }
    }

    fn calc_size(info: &TextureInfo) -> usize {
        let bytes_per_pixel = match info.format {
            Format::RGBA8 => 4,
            Format::RGB8 => 3,
            Format::R8 => 1,
            Format::RGBA16F => 8,
            Format::RGBA32F => 16,
            Format::D16 => 2,
            Format::D24S8 => 4,
            Format::D32F => 4,
            _ => 4,
        };
        info.width as usize * info.height as usize * info.depth as usize
            * info.array_layers as usize
            * bytes_per_pixel
    }

    pub fn get_width(&self) -> u32 {
        self.info.width
    }

    pub fn get_height(&self) -> u32 {
        self.info.height
    }

    pub fn get_depth(&self) -> u32 {
        self.info.depth
    }

    pub fn get_format(&self) -> Format {
        self.info.format
    }

    pub fn get_mip_levels(&self) -> u32 {
        self.info.mip_levels
    }

    pub fn get_array_layers(&self) -> u32 {
        self.info.array_layers
    }

    pub fn update(&mut self, data: &[u8], offset: usize) {
        let end = offset + data.len();
        if end <= self.data.len() {
            self.data[offset..end].copy_from_slice(data);
        }
    }

    pub fn new_view(id: u32, info: TextureViewInfo) -> Self {
        let view_info = TextureInfo {
            tex_type: info.tex_type,
            usage: TextureUsage::SAMPLED,
            format: info.format,
            width: 1,
            height: 1,
            depth: 1,
            array_layers: info.layer_count,
            mip_levels: info.level_count,
            ..Default::default()
        };
        GfxTexture {
            id,
            info: view_info,
            data: Vec::new(),
        }
    }

    pub fn is_view(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_new() {
        let info = TextureInfo {
            width: 256,
            height: 256,
            ..Default::default()
        };
        let tex = GfxTexture::new(1, info);
        assert_eq!(tex.get_width(), 256);
        assert_eq!(tex.get_height(), 256);
        assert_eq!(tex.get_format(), Format::RGBA8);
    }

    #[test]
    fn test_texture_info_default() {
        let info = TextureInfo::default();
        assert_eq!(info.tex_type, TextureType::Tex2D);
        assert_eq!(info.width, 1);
        assert_eq!(info.height, 1);
        assert_eq!(info.mip_levels, 1);
    }
}
