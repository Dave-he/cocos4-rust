/****************************************************************************
Rust port of Cocos Creator RenderTexture
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/
// SPDX-License-Identifier: MIT

use super::asset_enum::{Filter, PixelFormat, WrapMode};

#[derive(Debug, Clone, Default)]
pub struct IRenderTextureCreateInfo {
    pub name: Option<String>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct RenderTexture {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    pub min_filter: Filter,
    pub mag_filter: Filter,
    pub wrap_s: WrapMode,
    pub wrap_t: WrapMode,
    pub anisotropy: u32,
}

impl RenderTexture {
    pub fn new() -> Self {
        RenderTexture {
            width: 1,
            height: 1,
            format: PixelFormat::Rgba8888,
            min_filter: Filter::Linear,
            mag_filter: Filter::Linear,
            wrap_s: WrapMode::Repeat,
            wrap_t: WrapMode::Repeat,
            anisotropy: 0,
        }
    }

    pub fn initialize(&mut self, info: &IRenderTextureCreateInfo) {
        self.width = info.width;
        self.height = info.height;
        if let Some(name) = &info.name {
            self.format = PixelFormat::Rgba8888;
        }
    }

    pub fn reset(&mut self, info: &IRenderTextureCreateInfo) {
        self.initialize(info);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn read_pixels(&self, _x: u32, _y: u32, _w: u32, _h: u32) -> Vec<u8> {
        Vec::new()
    }
}

impl Default for RenderTexture {
    fn default() -> Self {
        RenderTexture::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_texture_new() {
        let rt = RenderTexture::new();
        assert_eq!(rt.width, 1);
        assert_eq!(rt.height, 1);
        assert_eq!(rt.format, PixelFormat::Rgba8888);
    }

    #[test]
    fn test_render_texture_initialize() {
        let mut rt = RenderTexture::new();
        let info = IRenderTextureCreateInfo {
            name: Some("test_rt".to_string()),
            width: 1024,
            height: 768,
        };
        rt.initialize(&info);
        assert_eq!(rt.width, 1024);
        assert_eq!(rt.height, 768);
    }

    #[test]
    fn test_render_texture_resize() {
        let mut rt = RenderTexture::new();
        rt.resize(512, 512);
        assert_eq!(rt.width, 512);
        assert_eq!(rt.height, 512);
    }

    #[test]
    fn test_render_texture_read_pixels() {
        let rt = RenderTexture::new();
        let pixels = rt.read_pixels(0, 0, 10, 10);
        assert!(pixels.is_empty());
    }
}
