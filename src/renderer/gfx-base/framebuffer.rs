/****************************************************************************
Rust port of Cocos Creator GFX Framebuffer
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

#[derive(Debug, Clone)]
pub struct FramebufferInfo {
    pub render_pass_id: u32,
    pub color_textures: Vec<u32>,
    pub depth_stencil_texture: Option<u32>,
    pub color_mipmaps: Vec<u32>,
    pub depth_stencil_mipmap: u32,
}

impl Default for FramebufferInfo {
    fn default() -> Self {
        FramebufferInfo {
            render_pass_id: 0,
            color_textures: Vec::new(),
            depth_stencil_texture: None,
            color_mipmaps: Vec::new(),
            depth_stencil_mipmap: 0,
        }
    }
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
        GfxFramebuffer { id, info, width, height }
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
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
