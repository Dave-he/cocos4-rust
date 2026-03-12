/****************************************************************************
Rust port of Cocos Creator GFX Swapchain
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::{Format, SampleCount, TextureUsage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceTransform {
    Identity = 0,
    Rotate90 = 1,
    Rotate180 = 2,
    Rotate270 = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VsyncMode {
    Off = 0,
    On = 1,
    Relaxed = 2,
    Mailbox = 3,
    Half = 4,
}

#[derive(Debug, Clone)]
pub struct SwapchainInfo {
    pub window_handle: u64,
    pub vsync_mode: VsyncMode,
    pub width: u32,
    pub height: u32,
}

impl Default for SwapchainInfo {
    fn default() -> Self {
        SwapchainInfo {
            window_handle: 0,
            vsync_mode: VsyncMode::On,
            width: 0,
            height: 0,
        }
    }
}

#[derive(Debug)]
pub struct GfxSwapchain {
    pub id: u32,
    pub info: SwapchainInfo,
    pub color_texture_id: u32,
    pub depth_stencil_texture_id: u32,
    pub surface_transform: SurfaceTransform,
}

impl GfxSwapchain {
    pub fn new(id: u32, info: SwapchainInfo) -> Self {
        GfxSwapchain {
            id,
            info,
            color_texture_id: 0,
            depth_stencil_texture_id: 0,
            surface_transform: SurfaceTransform::Identity,
        }
    }

    pub fn get_width(&self) -> u32 {
        self.info.width
    }

    pub fn get_height(&self) -> u32 {
        self.info.height
    }

    pub fn resize(&mut self, width: u32, height: u32, transform: SurfaceTransform) {
        self.info.width = width;
        self.info.height = height;
        self.surface_transform = transform;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swapchain_new() {
        let info = SwapchainInfo {
            width: 1920,
            height: 1080,
            ..Default::default()
        };
        let sc = GfxSwapchain::new(1, info);
        assert_eq!(sc.get_width(), 1920);
        assert_eq!(sc.get_height(), 1080);
    }

    #[test]
    fn test_swapchain_resize() {
        let mut sc = GfxSwapchain::new(1, SwapchainInfo::default());
        sc.resize(800, 600, SurfaceTransform::Identity);
        assert_eq!(sc.get_width(), 800);
        assert_eq!(sc.get_height(), 600);
    }
}
