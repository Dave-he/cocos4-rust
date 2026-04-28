/****************************************************************************
Rust port of Cocos Creator GFX Base System
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum API {
    Unknown = 0,
    GLES2 = 1,
    GLES3 = 2,
    Metal = 3,
    Vulkan = 4,
    NVN = 5,
    WebGL = 6,
    WebGL2 = 7,
    WebGPU = 8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    Unknown = 0,
    Swapchain,
    Buffer,
    Texture,
    RenderPass,
    Framebuffer,
    Sampler,
    Shader,
    DescriptorSetLayout,
    PipelineLayout,
    PipelineState,
    DescriptorSet,
    InputAssembler,
    CommandBuffer,
    Queue,
    QueryPool,
    GlobalBarrier,
    TextureBarrier,
    BufferBarrier,
    Count,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Unready = 0,
    Failed = 1,
    Success = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    ElementIndexUint = 0,
    InstancedArrays,
    MultipleRenderTargets,
    BlendMinmax,
    ComputeShader,
    SubpassColorInput,
    SubpassDepthStencilInput,
    RasterizationOrderNocoherent,
    MultiSampleResolveDepthStencil,
    Count,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatType {
    None = 0,
    Unorm,
    Snorm,
    Uint,
    Int,
    Ufloat,
    Float,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Unknown = 0,
    A8,
    L8,
    LA8,
    R8,
    R8SN,
    R8UI,
    R8I,
    R16F,
    R16UI,
    R16I,
    R32F,
    R32UI,
    R32I,
    RG8,
    RG8SN,
    RG8UI,
    RG8I,
    RG16F,
    RG16UI,
    RG16I,
    RG32F,
    RG32UI,
    RG32I,
    RGB8,
    SRGB8,
    RGB8SN,
    RGB8UI,
    RGB8I,
    RGB16F,
    RGB16UI,
    RGB16I,
    RGB32F,
    RGB32UI,
    RGB32I,
    RGBA8,
    BGRA8,
    SRGB8A8,
    RGBA8SN,
    RGBA8UI,
    RGBA8I,
    RGBA16F,
    RGBA16UI,
    RGBA16I,
    RGBA32F,
    RGBA32UI,
    RGBA32I,
    R5G6B5,
    R11G11B10F,
    RGB5A1,
    RGBA4,
    RGB10A2,
    RGB10A2UI,
    RGB9E5,
    Depth,
    DepthStencil,
    D16 = 9999,
    D24S8 = 10000,
    D32F = 10001,
    D32FS8 = 10002,
    X8D24 = 10003,
    BC1,
    BC1Alpha,
    BC1Srgb,
    BC1SrgbAlpha,
    BC2,
    BC2Srgb,
    BC3,
    BC3Srgb,
    BC4,
    BC4Snorm,
    BC5,
    BC5Snorm,
    BC6HUf16,
    BC6HSf16,
    BC7,
    BC7Srgb,
    EtcRgb8,
    Etc2Rgb8,
    Etc2Srgb8,
    Etc2Rgb8A1,
    Etc2Srgb8A1,
    Etc2Rgba8,
    Etc2Srgb8A8,
    EacR11,
    EacR11SN,
    EacRG11,
    EacRG11SN,
    PvrtcRgb2,
    PvrtcRgba2,
    PvrtcRgb4,
    PvrtcRgba4,
    Pvrtc22Bpp,
    Pvrtc24Bpp,
    AstcRgba4x4,
    AstcRgba5x4,
    AstcRgba5x5,
    AstcRgba6x5,
    AstcRgba6x6,
    AstcRgba8x5,
    AstcRgba8x6,
    AstcRgba8x8,
    AstcRgba10x5,
    AstcRgba10x6,
    AstcRgba10x8,
    AstcRgba10x10,
    AstcRgba12x10,
    AstcRgba12x12,
    AstcSrgba4x4,
    AstcSrgba5x4,
    AstcSrgba5x5,
    AstcSrgba6x5,
    AstcSrgba6x6,
    AstcSrgba8x5,
    AstcSrgba8x6,
    AstcSrgba8x8,
    AstcSrgba10x5,
    AstcSrgba10x6,
    AstcSrgba10x8,
    AstcSrgba10x10,
    AstcSrgba12x10,
    AstcSrgba12x12,
    Count,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FormatFeature(pub u32);

impl FormatFeature {
    pub const NONE: Self = Self(0);
    pub const RENDER_TARGET: Self = Self(1 << 0);
    pub const SAMPLED_TEXTURE: Self = Self(1 << 1);
    pub const LINEAR_FILTER: Self = Self(1 << 2);
    pub const STORAGE_TEXTURE: Self = Self(1 << 3);
    pub const VERTEX_ATTRIBUTE: Self = Self(1 << 4);
    pub const SHADING_RATE: Self = Self(1 << 5);

    pub fn contains(self, other: FormatFeature) -> bool {
        self.0 & other.0 == other.0
    }
}

impl std::ops::BitOr for FormatFeature {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferUsage(pub u32);

impl BufferUsage {
    pub const NONE: Self = Self(0);
    pub const TRANSFER_SRC: Self = Self(1 << 0);
    pub const TRANSFER_DST: Self = Self(1 << 1);
    pub const INDEX: Self = Self(1 << 2);
    pub const VERTEX: Self = Self(1 << 3);
    pub const UNIFORM: Self = Self(1 << 4);
    pub const STORAGE: Self = Self(1 << 5);
    pub const INDIRECT: Self = Self(1 << 6);

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl std::ops::BitOr for BufferUsage {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemoryUsage(pub u32);

impl MemoryUsage {
    pub const NONE: Self = Self(0);
    pub const DEVICE: Self = Self(1 << 0);
    pub const HOST: Self = Self(1 << 1);

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl std::ops::BitOr for MemoryUsage {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferFlags(pub u32);

impl BufferFlags {
    pub const NONE: Self = Self(0);
    pub const ENABLE_STAGING_WRITE: Self = Self(1 << 0);
}

impl std::ops::BitOr for BufferFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemoryAccess(pub u32);

impl MemoryAccess {
    pub const NONE: Self = Self(0);
    pub const READ_ONLY: Self = Self(1 << 0);
    pub const WRITE_ONLY: Self = Self(1 << 1);
    pub const READ_WRITE: Self = Self(0x1 | 0x2);
}

impl std::ops::BitOr for MemoryAccess {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureType {
    Tex1D = 0,
    Tex2D = 1,
    Tex3D = 2,
    Cube = 3,
    Tex1DArray = 4,
    Tex2DArray = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureUsage(pub u32);

impl TextureUsage {
    pub const NONE: Self = Self(0);
    pub const TRANSFER_SRC: Self = Self(1 << 0);
    pub const TRANSFER_DST: Self = Self(1 << 1);
    pub const SAMPLED: Self = Self(1 << 2);
    pub const STORAGE: Self = Self(1 << 3);
    pub const COLOR_ATTACHMENT: Self = Self(1 << 4);
    pub const DEPTH_STENCIL_ATTACHMENT: Self = Self(1 << 5);
    pub const INPUT_ATTACHMENT: Self = Self(1 << 6);
    pub const SHADING_RATE: Self = Self(1 << 7);

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl std::ops::BitOr for TextureUsage {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureFlags(pub u32);

impl TextureFlags {
    pub const NONE: Self = Self(0);
    pub const GEN_MIPMAP: Self = Self(1 << 0);
    pub const GENERAL_LAYOUT: Self = Self(1 << 1);
    pub const EXTERNAL_OES: Self = Self(1 << 2);
    pub const EXTERNAL_NORMAL: Self = Self(1 << 3);
    pub const LAZILY_ALLOCATED: Self = Self(1 << 4);
    pub const MUTABLE_VIEW_FORMAT: Self = Self(1 << 6);
    pub const MUTABLE_STORAGE: Self = Self(1 << 7);

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl std::ops::BitOr for TextureFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleCount {
    X1 = 0x01,
    X2 = 0x02,
    X4 = 0x04,
    X8 = 0x08,
    X16 = 0x10,
    X32 = 0x20,
    X64 = 0x40,
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
    Incr = 3,
    Decr = 4,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColorMask(pub u32);

impl ColorMask {
    pub const NONE: Self = Self(0x0);
    pub const R: Self = Self(0x1);
    pub const G: Self = Self(0x2);
    pub const B: Self = Self(0x4);
    pub const A: Self = Self(0x8);
    pub const ALL: Self = Self(0xF);
}

impl std::ops::BitOr for ColorMask {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolygonMode {
    Fill = 0,
    Point = 1,
    Line = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadeModel {
    Gourand = 0,
    Flat = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullMode {
    None = 0,
    Front = 1,
    Back = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveMode {
    PointList = 0,
    LineList = 1,
    LineStrip = 2,
    LineLoop = 3,
    LineListAdjacency = 4,
    LineStripAdjacency = 5,
    IsoLineList = 6,
    TriangleList = 7,
    TriangleStrip = 8,
    TriangleFan = 9,
    TriangleListAdjacency = 10,
    TriangleStripAdjacency = 11,
    TrianglePatchAdjacency = 12,
    QuadPatchList = 13,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DynamicStateFlags(pub u32);

impl DynamicStateFlags {
    pub const NONE: Self = Self(0x0);
    pub const LINE_WIDTH: Self = Self(0x1);
    pub const DEPTH_BIAS: Self = Self(0x2);
    pub const BLEND_CONSTANTS: Self = Self(0x4);
    pub const DEPTH_BOUNDS: Self = Self(0x8);
    pub const STENCIL_WRITE_MASK: Self = Self(0x10);
    pub const STENCIL_COMPARE_MASK: Self = Self(0x20);
}

impl std::ops::BitOr for DynamicStateFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StencilFace(pub u32);

impl StencilFace {
    pub const FRONT: Self = Self(0x1);
    pub const BACK: Self = Self(0x2);
    pub const ALL: Self = Self(0x3);
}

impl std::ops::BitOr for StencilFace {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShaderStageFlags(pub u32);

impl ShaderStageFlags {
    pub const NONE: Self = Self(0x0);
    pub const VERTEX: Self = Self(0x1);
    pub const CONTROL: Self = Self(0x2);
    pub const EVALUATION: Self = Self(0x4);
    pub const GEOMETRY: Self = Self(0x8);
    pub const FRAGMENT: Self = Self(0x10);
    pub const COMPUTE: Self = Self(0x20);
    pub const ALL: Self = Self(0x3f);
}

impl std::ops::BitOr for ShaderStageFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

pub type ShaderStageFlagBit = ShaderStageFlags;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccessFlags(pub u64);

impl AccessFlags {
    pub const NONE: Self = Self(0);
    pub const INDIRECT_BUFFER: Self = Self(1 << 0);
    pub const INDEX_BUFFER: Self = Self(1 << 1);
    pub const VERTEX_BUFFER: Self = Self(1 << 2);
    pub const VERTEX_SHADER_READ_UNIFORM_BUFFER: Self = Self(1 << 3);
    pub const VERTEX_SHADER_READ_TEXTURE: Self = Self(1 << 4);
    pub const VERTEX_SHADER_READ_OTHER: Self = Self(1 << 5);
    pub const FRAGMENT_SHADER_READ_UNIFORM_BUFFER: Self = Self(1 << 6);
    pub const FRAGMENT_SHADER_READ_TEXTURE: Self = Self(1 << 7);
    pub const FRAGMENT_SHADER_READ_COLOR_INPUT_ATTACHMENT: Self = Self(1 << 8);
    pub const FRAGMENT_SHADER_READ_DEPTH_STENCIL_INPUT_ATTACHMENT: Self = Self(1 << 9);
    pub const FRAGMENT_SHADER_READ_OTHER: Self = Self(1 << 10);
    pub const COLOR_ATTACHMENT_READ: Self = Self(1 << 11);
    pub const DEPTH_STENCIL_ATTACHMENT_READ: Self = Self(1 << 12);
    pub const COMPUTE_SHADER_READ_UNIFORM_BUFFER: Self = Self(1 << 13);
    pub const COMPUTE_SHADER_READ_TEXTURE: Self = Self(1 << 14);
    pub const COMPUTE_SHADER_READ_OTHER: Self = Self(1 << 15);
    pub const TRANSFER_READ: Self = Self(1 << 16);
    pub const HOST_READ: Self = Self(1 << 17);
    pub const PRESENT: Self = Self(1 << 18);
    pub const VERTEX_SHADER_WRITE: Self = Self(1 << 19);
    pub const FRAGMENT_SHADER_WRITE: Self = Self(1 << 20);
    pub const COLOR_ATTACHMENT_WRITE: Self = Self(1 << 21);
    pub const DEPTH_STENCIL_ATTACHMENT_WRITE: Self = Self(1 << 22);
    pub const COMPUTE_SHADER_WRITE: Self = Self(1 << 23);
    pub const TRANSFER_WRITE: Self = Self(1 << 24);
    pub const HOST_PREINITIALIZED: Self = Self(1 << 25);
    pub const HOST_WRITE: Self = Self(1 << 26);
    pub const SHADING_RATE: Self = Self(1 << 27);

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl std::ops::BitOr for AccessFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolveMode {
    None = 0,
    SampleZero = 1,
    Average = 2,
    Min = 3,
    Max = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineBindPoint {
    Graphics = 0,
    Compute = 1,
    RayTracing = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueType {
    Graphics = 0,
    Compute = 1,
    Transfer = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    Occlusion = 0,
    PipelineStatistics = 1,
    Timestamp = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandBufferType {
    Primary = 0,
    Secondary = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClearFlags(pub u32);

impl ClearFlags {
    pub const NONE: Self = Self(0);
    pub const COLOR: Self = Self(0x1);
    pub const DEPTH: Self = Self(0x2);
    pub const STENCIL: Self = Self(0x4);
    pub const DEPTH_STENCIL: Self = Self(0x2 | 0x4);
    pub const ALL: Self = Self(0x1 | 0x2 | 0x4);
}

impl std::ops::BitOr for ClearFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarrierType {
    Full = 0,
    SplitBegin = 1,
    SplitEnd = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassType {
    Raster = 0,
    Compute = 1,
    Copy = 2,
    Move = 3,
    Raytrace = 4,
    Present = 5,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Offset {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Extent {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl Extent {
    pub fn new(width: u32, height: u32, depth: u32) -> Self {
        Extent { width, height, depth }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Viewport {
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
    pub min_depth: f32,
    pub max_depth: f32,
}

impl Viewport {
    pub fn new(left: i32, top: i32, width: u32, height: u32) -> Self {
        Viewport { left, top, width, height, min_depth: 0.0, max_depth: 1.0 }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Color {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Color {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Color { x, y, z, w }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MarkerInfo {
    pub name: String,
    pub color: Color,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TextureSubresLayers {
    pub mip_level: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
}

impl TextureSubresLayers {
    pub fn new(mip_level: u32, base_array_layer: u32, layer_count: u32) -> Self {
        TextureSubresLayers { mip_level, base_array_layer, layer_count }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TextureSubresRange {
    pub base_mip_level: u32,
    pub level_count: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TextureCopy {
    pub src_subres: TextureSubresLayers,
    pub src_offset: Offset,
    pub dst_subres: TextureSubresLayers,
    pub dst_offset: Offset,
    pub extent: Extent,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TextureBlit {
    pub src_subres: TextureSubresLayers,
    pub src_offset: Offset,
    pub src_extent: Extent,
    pub dst_subres: TextureSubresLayers,
    pub dst_offset: Offset,
    pub dst_extent: Extent,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BufferTextureCopy {
    pub buff_offset: u32,
    pub buff_stride: u32,
    pub buff_tex_height: u32,
    pub tex_offset: Offset,
    pub tex_extent: Extent,
    pub tex_subres: TextureSubresLayers,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DrawInfo {
    pub vertex_count: u32,
    pub first_vertex: u32,
    pub index_count: u32,
    pub first_index: u32,
    pub vertex_offset: i32,
    pub instance_count: u32,
    pub first_instance: u32,
}

#[derive(Debug, Clone, Default)]
pub struct DispatchInfo {
    pub group_count_x: u32,
    pub group_count_y: u32,
    pub group_count_z: u32,
    pub indirect_buffer: Option<u32>,
    pub indirect_offset: u32,
}

#[derive(Debug, Clone, Default)]
pub struct Size {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

#[derive(Debug, Clone, Default)]
pub struct DeviceCaps {
    pub max_vertex_attributes: u32,
    pub max_vertex_uniform_vectors: u32,
    pub max_fragment_uniform_vectors: u32,
    pub max_texture_units: u32,
    pub max_image_units: u32,
    pub max_vertex_texture_units: u32,
    pub max_color_render_targets: u32,
    pub max_shader_storage_buffer_bindings: u32,
    pub max_shader_storage_block_size: u32,
    pub max_uniform_buffer_bindings: u32,
    pub max_uniform_block_size: u32,
    pub max_texture_size: u32,
    pub max_cube_map_texture_size: u32,
    pub max_array_texture_layers: u32,
    pub max_3d_texture_size: u32,
    pub ubo_offset_alignment: u32,
    pub max_compute_shared_memory_size: u32,
    pub max_compute_work_group_invocations: u32,
    pub max_compute_work_group_size: Size,
    pub max_compute_work_group_count: Size,
    pub support_query: bool,
    pub support_variable_rate_shading: bool,
    pub support_sub_pass_shading: bool,
    pub clip_space_min_z: f32,
    pub screen_space_sign_y: f32,
    pub clip_space_sign_y: f32,
}

impl DeviceCaps {
    pub fn new() -> Self {
        DeviceCaps {
            ubo_offset_alignment: 1,
            clip_space_min_z: -1.0,
            screen_space_sign_y: 1.0,
            clip_space_sign_y: 1.0,
            ..Default::default()
        }
    }
}

pub const MAX_ATTACHMENTS: u32 = 4;
pub const INVALID_BINDING: u32 = !0u32;
pub const SUBPASS_EXTERNAL: u32 = !0u32;

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
