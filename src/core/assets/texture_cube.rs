/****************************************************************************
Rust port of Cocos Creator TextureCube
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/
// SPDX-License-Identifier: MIT

use super::asset_enum::{Filter, PixelFormat, WrapMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MipmapMode {
    None = 0,
    Auto = 1,
    BakedConvolutionMap = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceIndex {
    Right = 0,
    Left = 1,
    Top = 2,
    Bottom = 3,
    Front = 4,
    Back = 5,
}

#[derive(Debug, Default)]
pub struct ITextureCubeMipmap {
    pub front: Option<String>,
    pub back: Option<String>,
    pub left: Option<String>,
    pub right: Option<String>,
    pub top: Option<String>,
    pub bottom: Option<String>,
}

#[derive(Debug)]
pub struct TextureCube {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    pub mipmap_mode: MipmapMode,
    pub is_rgbe: bool,
    pub min_filter: Filter,
    pub mag_filter: Filter,
    pub wrap_s: WrapMode,
    pub wrap_t: WrapMode,
    pub mipmaps: Vec<ITextureCubeMipmap>,
}

impl TextureCube {
    pub fn new() -> Self {
        TextureCube {
            width: 1,
            height: 1,
            format: PixelFormat::Rgba8888,
            mipmap_mode: MipmapMode::None,
            is_rgbe: false,
            min_filter: Filter::Linear,
            mag_filter: Filter::Linear,
            wrap_s: WrapMode::Repeat,
            wrap_t: WrapMode::Repeat,
            mipmaps: Vec::new(),
        }
    }

    pub fn set_mipmaps(&mut self, mipmaps: Vec<ITextureCubeMipmap>) {
        self.mipmaps = mipmaps;
    }

    pub fn get_image(&self) -> Option<&ITextureCubeMipmap> {
        self.mipmaps.first()
    }

    pub fn reset(&mut self) {
        self.mipmaps.clear();
        self.mipmap_mode = MipmapMode::None;
    }
}

impl Default for TextureCube {
    fn default() -> Self {
        TextureCube::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_cube_new() {
        let tc = TextureCube::new();
        assert_eq!(tc.width, 1);
        assert_eq!(tc.height, 1);
        assert_eq!(tc.mipmap_mode, MipmapMode::None);
        assert!(!tc.is_rgbe);
        assert!(tc.mipmaps.is_empty());
    }

    #[test]
    fn test_texture_cube_set_mipmaps() {
        let mut tc = TextureCube::new();
        tc.set_mipmaps(vec![ITextureCubeMipmap::default()]);
        assert_eq!(tc.mipmaps.len(), 1);
    }

    #[test]
    fn test_face_index_values() {
        assert_eq!(FaceIndex::Right as u32, 0);
        assert_eq!(FaceIndex::Back as u32, 5);
    }

    #[test]
    fn test_mipmap_mode_values() {
        assert_eq!(MipmapMode::None as u32, 0);
        assert_eq!(MipmapMode::Auto as u32, 1);
        assert_eq!(MipmapMode::BakedConvolutionMap as u32, 2);
    }
}
