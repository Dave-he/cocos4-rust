/****************************************************************************
Rust port of Cocos Creator GFX Base System
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

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
    RGB32F,
    RGB32I,
    RGB32UI,
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
    BC1,
    BC2,
    BC3,
    COUNT,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferUsage(u32);

impl BufferUsage {
    pub const NONE: BufferUsage = BufferUsage(0);
    pub const TRANSFER_SRC: BufferUsage = BufferUsage(1 << 0);
    pub const TRANSFER_DST: BufferUsage = BufferUsage(1 << 1);
    pub const INDEX: BufferUsage = BufferUsage(1 << 2);
    pub const VERTEX: BufferUsage = BufferUsage(1 << 3);
    pub const UNIFORM: BufferUsage = BufferUsage(1 << 4);
    pub const STORAGE: BufferUsage = BufferUsage(1 << 5);
    pub const INDIRECT: BufferUsage = BufferUsage(1 << 6);

    pub fn contains(self, other: BufferUsage) -> bool {
        self.0 & other.0 == other.0
    }
}

impl std::ops::BitOr for BufferUsage {
    type Output = BufferUsage;
    fn bitor(self, rhs: Self) -> Self::Output {
        BufferUsage(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemoryUsage(u32);

impl MemoryUsage {
    pub const NONE: MemoryUsage = MemoryUsage(0);
    pub const DEVICE: MemoryUsage = MemoryUsage(1 << 0);
    pub const HOST: MemoryUsage = MemoryUsage(1 << 1);

    pub fn contains(self, other: MemoryUsage) -> bool {
        self.0 & other.0 == other.0
    }
}

impl std::ops::BitOr for MemoryUsage {
    type Output = MemoryUsage;
    fn bitor(self, rhs: Self) -> Self::Output {
        MemoryUsage(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferFlags(u32);

impl BufferFlags {
    pub const NONE: BufferFlags = BufferFlags(0);
    pub const INDIRECT: BufferFlags = BufferFlags(1 << 0);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureType {
    Tex1D = 0,
    Tex2D = 1,
    Tex3D = 2,
    Cube = 3,
    Tex1DArray = 4,
    Tex2DArray = 5,
    CubeArray = 6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureUsage(u32);

impl TextureUsage {
    pub const NONE: TextureUsage = TextureUsage(0);
    pub const TRANSFER_SRC: TextureUsage = TextureUsage(1 << 0);
    pub const TRANSFER_DST: TextureUsage = TextureUsage(1 << 1);
    pub const SAMPLED: TextureUsage = TextureUsage(1 << 2);
    pub const STORAGE: TextureUsage = TextureUsage(1 << 3);
    pub const COLOR_ATTACHMENT: TextureUsage = TextureUsage(1 << 4);
    pub const DEPTH_STENCIL_ATTACHMENT: TextureUsage = TextureUsage(1 << 5);
    pub const INPUT_ATTACHMENT: TextureUsage = TextureUsage(1 << 6);
    pub const SHADING_RATE: TextureUsage = TextureUsage(1 << 7);
}

impl std::ops::BitOr for TextureUsage {
    type Output = TextureUsage;
    fn bitor(self, rhs: Self) -> Self::Output {
        TextureUsage(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureFlags(u32);

impl TextureFlags {
    pub const NONE: TextureFlags = TextureFlags(0);
    pub const GEN_MIPMAP: TextureFlags = TextureFlags(1 << 0);
    pub const GENERAL_LAYOUT: TextureFlags = TextureFlags(1 << 1);
    pub const EXTERNAL_OES: TextureFlags = TextureFlags(1 << 2);
    pub const EXTERNAL_NORMAL: TextureFlags = TextureFlags(1 << 3);
    pub const LAZILY_ALLOCATED: TextureFlags = TextureFlags(1 << 4);
    pub const MUTABLE_VIEW_FORMAT: TextureFlags = TextureFlags(1 << 5);
}

impl std::ops::BitOr for TextureFlags {
    type Output = TextureFlags;
    fn bitor(self, rhs: Self) -> Self::Output {
        TextureFlags(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleCount {
    X1 = 1,
    X2 = 2,
    X4 = 4,
    X8 = 8,
    X16 = 16,
    X32 = 32,
    X64 = 64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    None = 0,
    Point = 1,
    Linear = 2,
    Anisotropic = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Address {
    Wrap = 0,
    Mirror = 1,
    Clamp = 2,
    Border = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonFunc {
    Never = 0,
    Less = 1,
    Equal = 2,
    LessEqual = 3,
    Greater = 4,
    NotEqual = 5,
    GreaterEqual = 6,
    Always = 7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StencilOp {
    Zero = 0,
    Keep = 1,
    Replace = 2,
    IncrSat = 3,
    DecrSat = 4,
    Invert = 5,
    IncrWrap = 6,
    DecrWrap = 7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendOp {
    Add = 0,
    Sub = 1,
    RevSub = 2,
    Min = 3,
    Max = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendFactor {
    Zero = 0,
    One = 1,
    SrcAlpha = 2,
    DstAlpha = 3,
    OneMinusSrcAlpha = 4,
    OneMinusDstAlpha = 5,
    SrcColor = 6,
    DstColor = 7,
    OneMinusSrcColor = 8,
    OneMinusDstColor = 9,
    SrcAlphaSaturate = 10,
    ConstantColor = 11,
    OneMinusConstantColor = 12,
    ConstantAlpha = 13,
    OneMinusConstantAlpha = 14,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveMode {
    PointList = 0,
    LineList = 1,
    LineStrip = 2,
    LineLoop = 3,
    TriangleList = 4,
    TriangleStrip = 5,
    TriangleFan = 6,
    LineListAdjacency = 7,
    LineStripAdjacency = 8,
    TriangleListAdjacency = 9,
    TriangleStripAdjacency = 10,
    TrianglePatch = 11,
    QuadPatch = 12,
    IsoPatch = 13,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderStageFlagBit {
    None = 0,
    Vertex = 1,
    Control = 2,
    Evaluation = 4,
    Geometry = 8,
    Fragment = 16,
    Compute = 32,
    All = 63,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadOp {
    Load = 0,
    Clear = 1,
    Discard = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreOp {
    Store = 0,
    Discard = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueType {
    Graphics = 0,
    Compute = 1,
    Transfer = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandBufferType {
    Primary = 0,
    Secondary = 1,
}

#[derive(Debug, Clone)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Default for Rect {
    fn default() -> Self {
        Rect { x: 0, y: 0, width: 0, height: 0 }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Viewport {
            left: 0.0,
            top: 0.0,
            width: 0.0,
            height: 0.0,
            min_depth: 0.0,
            max_depth: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for Color {
    fn default() -> Self {
        Color { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }
    }
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
