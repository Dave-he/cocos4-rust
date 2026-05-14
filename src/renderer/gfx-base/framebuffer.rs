/****************************************************************************
Rust port of Cocos Creator GFX Framebuffer
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

#[derive(Debug, Clone, Default)]
pub struct FramebufferInfo {
    pub render_pass_id: u32,
    pub color_textures: Vec<u32>,
    pub depth_stencil_texture: Option<u32>,
    pub color_mipmaps: Vec<u32>,
    pub depth_stencil_mipmap: u32,
}

#[derive(Debug)]
pub struct GfxFramebuffer {
    pub id: u32,
    pub info: FramebufferInfo,
    pub width: u32,
    pub height: u32,
}

impl GfxFramebuffer {
    pub fn new(id: u32, info: FramebufferInfo, width: u32, height: u32) -> Self {
        GfxFramebuffer {
            id,
            info,
            width,
            height,
        }
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn get_render_pass_id(&self) -> u32 {
        self.info.render_pass_id
    }

    pub fn get_color_texture_count(&self) -> usize {
        self.info.color_textures.len()
    }

    pub fn destroy(&mut self) {
        self.info.color_textures.clear();
        self.info.depth_stencil_texture = None;
    }

    pub fn get_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        self.id.hash(&mut h);
        self.info.render_pass_id.hash(&mut h);
        for &t in &self.info.color_textures {
            t.hash(&mut h);
        }
        self.info.depth_stencil_texture.hash(&mut h);
        h.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framebuffer_new() {
        let info = FramebufferInfo::default();
        let fb = GfxFramebuffer::new(1, info, 1920, 1080);
        assert_eq!(fb.get_width(), 1920);
        assert_eq!(fb.get_height(), 1080);
    }
}
