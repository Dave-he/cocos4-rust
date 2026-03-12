/****************************************************************************
Rust port of Cocos Creator GFX Base System
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::RefCounted;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum API {
    GL = 0,
    GLES1 = 1,
    GLES2 = 2,
    GLES3 = 3,
    Vulkan = 4,
    Metal = 5,
    WebGPU = 6,
    WebGL2 = 7,
    DirectX12 = 8,
    Bus = 9,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Unknown = 0,
    A8,
    L8,
    LA8,
    R8,
    R8I,
    R8UI,
    R8SN,
    R16F,
    R16I,
    R16UI,
    RG8,
    RG8I,
    RG8UI,
    RG8SN,
    RG16F,
    RG16I,
    RG16UI,
    RGB8,
    RGB8I,
    RGB8UI,
    RGB8SN,
    RGB9E5,
    RGBA8,
    RGBA8I,
    RGBA8UI,
    RGBA8SN,
    RGBA16F,
    RGBA16I,
    RGBA16UI,
    RGB10A2,
    RGB10A2UI,
    RG11B10F,
    RG32F,
    RG32I,
    RG32UI,
    RGBA32F,
    RGBA32I,
    RGBA32UI,
    BGRA8,
    RGBX8,
    R5G6B5,
    R11G11B10F,
    RGB5A1,
    RGBA4,
    D16,
    D24S8,
    D32F,
    D32FS8,
    X8D24,
    AstcRgba4x4,
    AstcRgba5x4,
    AstcRgba5x5,
    AstcRgba6x5,
    AstcRgba6x6,
    AstcRgba7x6,
    AstcRgba8x5,
    AstcRgba8x6,
    AstcRgba8x8,
    AstcRgba9x5,
    AstcRgba9x6,
    AstcRgba10x5,
    AstcRgba10x6,
    AstcRgba10x8,
    AstcRgba10x10,
    AstcRgba11x11,
    AstcRgba12x10,
    AstcRgba12x12,
    AstcSrgb8A84x4,
    AstcSrgb8A85x4,
    AstcSrgb8A85x5,
    AstcSrgb8A86x5,
    AstcSrgb8A86x6,
    AstcSrgb8A87x6,
    AstcSrgb8A88x5,
    AstcSrgb8A88x6,
    ASTC_SRGB8_A8_8x8,
    ASTC_SRGB8_A8_9x5,
    ASTC_SRGB8_A8_9x6,
    ASTC_SRGB8_A8_10x5,
    ASTC_SRGB8_A8_10x6,
    ASTC_SRGB8_A8_10x8,
    ASTC_SRGB8_A8_10x10,
    ASTC_SRGB8_A8_11x11,
    ASTC_SRGB8_A8_12x10,
    ASTC_SRGB8_A8_12x12,
    BC1,
    BC2,
    BC3,
    BC4,
    BC5,
    BC6H,
    BC7,
    ETC,
    ETC2,
    EAC,
    PVRTC,
    PVRTC2,
    ATC,
    ATCA,
    ATCI,
    S3TC,
    S3TCI,
    S3TC_BGRA,
    I8,
    AI8,
    A16I,
    A16F,
    AI16F,
    L16F,
    L16I,
    LA8I,
    LA16F,
    LA16I,
    RGBX8UI,
    RGBX8I,
    RGB16F,
    RGB16I,
    RGB16UI,
    RGBA16,
    RGBA16SN,
    RGB32F,
    RGB32I,
    RGB32UI,
    R32F,
    R32UI,
    COUNT,
}

pub mod barrier;
pub mod buffer;
pub mod command_buffer;
pub mod descriptor;
pub mod device;
pub mod framebuffer;
pub mod input_assembler;
pub mod pipeline;
pub mod query;
pub mod queue;
pub mod render_pass;
pub mod sampler;
pub mod shader;
pub mod state;
pub mod swapchain;
pub mod texture;

pub use barrier::*;
pub use buffer::*;
pub use command_buffer::*;
pub use descriptor::*;
pub use device::*;
pub use framebuffer::*;
pub use input_assembler::*;
pub use pipeline::*;
pub use query::*;
pub use queue::*;
pub use render_pass::*;
pub use sampler::*;
pub use shader::*;
pub use state::*;
pub use swapchain::*;
pub use texture::*;
