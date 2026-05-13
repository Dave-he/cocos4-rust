/****************************************************************************
Rust port of Cocos Creator Pipeline Define System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::math::Vec3;
use crate::math::Vec4;
use crate::math::Color;
use crate::renderer::gfx_base::{Format, TextureType, TextureUsage};

pub const SHADOW_CAMERA_MAX_FAR: f32 = 2000.0;
pub const SKINNING_JOINT_UNIFORM_CAPACITY: u32 = 30;

pub const PIPELINE_FLOW_MAIN: &str = "MainFlow";
pub const PIPELINE_FLOW_FORWARD: &str = "ForwardFlow";
pub const PIPELINE_FLOW_SHADOW: &str = "ShadowFlow";
pub const PIPELINE_FLOW_SMAA: &str = "SMAAFlow";
pub const PIPELINE_FLOW_TONEMAP: &str = "ToneMapFlow";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderPassStage {
    #[default]
    DEFAULT = 100,
    UI = 200,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPriority {
    MIN = 0,
    DEFAULT = 0x80,
    MAX = 0xff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum PipelineGlobalBindings {
    UBO_GLOBAL = 0,
    UBO_CAMERA = 1,
    UBO_SHADOW = 2,
    UBO_CSM = 3,
    SAMPLER_SHADOWMAP = 4,
    SAMPLER_ENVIRONMENT = 5,
    SAMPLER_SPOT_SHADOW_MAP = 6,
    SAMPLER_DIFFUSEMAP = 7,
    COUNT = 8,
}

pub const GLOBAL_UBO_COUNT: u32 = PipelineGlobalBindings::SAMPLER_SHADOWMAP as u32;
pub const GLOBAL_SAMPLER_COUNT: u32 = PipelineGlobalBindings::COUNT as u32 - GLOBAL_UBO_COUNT;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum ModelLocalBindings {
    UBO_LOCAL = 0,
    UBO_FORWARD_LIGHTS = 1,
    UBO_SKINNING_ANIMATION = 2,
    UBO_SKINNING_TEXTURE = 3,
    UBO_MORPH = 4,
    UBO_UI_LOCAL = 5,
    UBO_SH = 6,
    SAMPLER_JOINTS = 7,
    SAMPLER_MORPH_POSITION = 8,
    SAMPLER_MORPH_NORMAL = 9,
    SAMPLER_MORPH_TANGENT = 10,
    SAMPLER_LIGHTMAP = 11,
    SAMPLER_SPRITE = 12,
    SAMPLER_REFLECTION_PROBE_CUBE = 13,
    SAMPLER_REFLECTION_PROBE_PLANAR = 14,
    SAMPLER_REFLECTION_PROBE_DATA_MAP = 15,
    COUNT = 16,
}

pub const LOCAL_UBO_COUNT: u32 = ModelLocalBindings::SAMPLER_JOINTS as u32;
pub const LOCAL_SAMPLER_COUNT: u32 = ModelLocalBindings::COUNT as u32 - LOCAL_UBO_COUNT;
pub const LOCAL_STORAGE_IMAGE_COUNT: u32 = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetIndex {
    GLOBAL = 0,
    MATERIAL = 1,
    LOCAL = 2,
    COUNT = 3,
}

pub const SET_INDEX_GLOBAL: u32 = SetIndex::GLOBAL as u32;
pub const SET_INDEX_MATERIAL: u32 = SetIndex::MATERIAL as u32;
pub const SET_INDEX_LOCAL: u32 = SetIndex::LOCAL as u32;

pub const UBO_GLOBAL_COUNT: u32 = 22;
pub const UBO_CAMERA_COUNT: u32 = 40;
pub const UBO_SHADOW_COUNT: u32 = 32;
pub const UBO_CSM_COUNT: u32 = 4;

pub const UBO_GLOBAL_FLOAT_OFFSET: u32 = 0;
pub const UBO_GLOBAL_MAT_OFFSET: u32 = 4;

pub const UBO_CAMERA_MAT_OFFSET: u32 = 0;
pub const UBO_CAMERA_POS_OFFSET: u32 = 16;
pub const UBO_CAMERA_DIR_OFFSET: u32 = 20;
pub const UBO_CAMERA_COLOR_OFFSET: u32 = 24;

pub const UBO_SHADOW_MAT_OFFSET: u32 = 0;
pub const UBO_SHADOW_INFO_OFFSET: u32 = 16;
pub const UBO_SHADOW_LIGHT_INFO_OFFSET: u32 = 20;

pub const INJECT_VFX_MACRO_STRING: &str = "#define CC_VFX_VERTEX 1\n";
pub const STANDARD_EXIT_MACRO_STRING: &str = "";

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct ClearFlagBit: u32 {
        const NONE = 0;
        const COLOR = 0x1;
        const DEPTH = 0x2;
        const STENCIL = 0x4;
        const ALL = 0x7;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct VisibilityFlags: u32 {
        const DEFAULT = 0x1;
        const UI = 0x2;
        const GIZMO = 0x4;
        const EDITOR = 0x8;
        const SCENE_GIZMO = 0x10;
        const PROFILER = 0x20;
        const ALL = 0xFFFFFFFF;
    }
}

#[derive(Debug, Clone, Default)]
pub struct RenderObject {
    pub depth: f32,
    pub model_id: u64,
}

#[derive(Debug, Clone)]
pub struct RenderPassItem {
    pub priority: i32,
    pub hash: u32,
    pub depth: f32,
    pub shader_id: u32,
    pub sub_model_index: u32,
    pub pass_index: u32,
}

impl Default for RenderPassItem {
    fn default() -> Self {
        RenderPassItem {
            priority: 0,
            hash: 0,
            depth: 0.0,
            shader_id: 0,
            sub_model_index: 0,
            pass_index: 0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RenderBatch {
    pub pass_index: u32,
}

#[derive(Debug, Clone, Default)]
pub struct RenderPassDesc {
    pub index: u32,
}

#[derive(Debug, Clone)]
pub struct RenderQueueDesc {
    pub is_transparent: bool,
    pub phases: u32,
    pub sort_mode: SortingOrder,
}

impl Default for RenderQueueDesc {
    fn default() -> Self {
        RenderQueueDesc {
            is_transparent: false,
            phases: 0,
            sort_mode: SortingOrder::FrontToBack,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderTextureDesc {
    pub name: String,
    pub tex_type: TextureType,
    pub usage: TextureUsage,
    pub format: Format,
    pub width: i32,
    pub height: i32,
}

impl Default for RenderTextureDesc {
    fn default() -> Self {
        RenderTextureDesc {
            name: String::new(),
            tex_type: TextureType::Tex2D,
            usage: TextureUsage::COLOR_ATTACHMENT,
            format: Format::Unknown,
            width: -1,
            height: -1,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FrameBufferDesc {
    pub name: String,
    pub render_pass: u32,
    pub color_textures: Vec<String>,
    pub depth_stencil_texture: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderFlowType {
    Scene = 0,
    Postprocess = 1,
    UI = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefineType {
    Int = 0,
    Bool = 1,
    String = 2,
    Number = 3,
    Buffer = 4,
}

#[derive(Debug, Clone)]
pub struct MacroRecord {
    pub name: String,
    pub value: String,
}

impl Default for MacroRecord {
    fn default() -> Self {
        MacroRecord {
            name: String::new(),
            value: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MacroValue {
    pub define_type: DefineType,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightType {
    Directional = 0,
    Point = 1,
    Spot = 2,
    RangedDirectional = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowType {
    None = 0,
    Planar = 1,
    ShadowMap = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PCFType {
    None = 0,
    Hard = 1,
    Soft = 2,
    Soft2 = 3,
    Soft3 = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CSMLevel {
    Level1 = 1,
    #[default]
    Level2 = 2,
    Level3 = 3,
    Level4 = 4,
}

#[derive(Debug, Clone)]
pub struct LightInfo {
    pub light: Option<()>,
    pub priority: i32,
    pub stage: u32,
}

impl LightInfo {
    pub fn new() -> Self {
        LightInfo {
            light: None,
            priority: 0,
            stage: 0,
        }
    }
}

impl Default for LightInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct DirectionalLightInfo {
    pub color: Color,
    pub direction: Vec3,
    pub intensity: f32,
    pub shadow_enabled: bool,
    pub shadow_pcf: PCFType,
    pub shadow_bias: f32,
    pub shadow_normal_bias: f32,
    pub shadow_distance: f32,
    pub shadow_ortho_size: f32,
}

impl Default for DirectionalLightInfo {
    fn default() -> Self {
        DirectionalLightInfo {
            color: Color::WHITE,
            direction: Vec3::new(0.0, -1.0, 0.0),
            intensity: 1.0,
            shadow_enabled: false,
            shadow_pcf: PCFType::Soft2,
            shadow_bias: 0.00001,
            shadow_normal_bias: 0.0,
            shadow_distance: 50.0,
            shadow_ortho_size: 5.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PointLightInfo {
    pub color: Color,
    pub position: Vec3,
    pub intensity: f32,
    pub range: f32,
    pub decay: f32,
}

impl Default for PointLightInfo {
    fn default() -> Self {
        PointLightInfo {
            color: Color::WHITE,
            position: Vec3::ZERO,
            intensity: 1.0,
            range: 1.0,
            decay: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpotLightInfo {
    pub color: Color,
    pub position: Vec3,
    pub direction: Vec3,
    pub intensity: f32,
    pub range: f32,
    pub spot_angle: f32,
    pub spot_exponent: f32,
    pub penumbra: f32,
    pub decay: f32,
}

impl Default for SpotLightInfo {
    fn default() -> Self {
        SpotLightInfo {
            color: Color::WHITE,
            position: Vec3::ZERO,
            direction: Vec3::new(0.0, -1.0, 0.0),
            intensity: 1.0,
            range: 1.0,
            spot_angle: 30.0,
            spot_exponent: 1.0,
            penumbra: 0.0,
            decay: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraProjection {
    #[default]
    Perspective = 0,
    Ortho = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraFOV {
    #[default]
    Vertical = 0,
    Horizontal = 1,
}

#[derive(Debug, Clone)]
pub struct CameraInfo {
    pub projection: CameraProjection,
    pub fov: f32,
    pub fov_axis: CameraFOV,
    pub aspect_ratio: f32,
    pub ortho_height: f32,
    pub near: f32,
    pub far: f32,
    pub color: Color,
    pub depth: i32,
    pub stencil: i32,
    pub clear_flags: u32,
    pub rect: Vec4,
}

impl Default for CameraInfo {
    fn default() -> Self {
        CameraInfo {
            projection: CameraProjection::Perspective,
            fov: 60.0,
            fov_axis: CameraFOV::Vertical,
            aspect_ratio: 16.0 / 9.0,
            ortho_height: 10.0,
            near: 0.1,
            far: 1000.0,
            color: Color::BLACK,
            depth: 1,
            stencil: 0,
            clear_flags: 0,
            rect: Vec4::new(0.0, 0.0, 1.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortingOrder {
    #[default]
    FrontToBack,
    BackToFront,
    ByPriority,
}

pub struct UBOGlobal;
impl UBOGlobal {
    pub const TIME_OFFSET: u32 = 0;
    pub const SCREEN_SIZE_OFFSET: u32 = 4;
    pub const NATIVE_SIZE_OFFSET: u32 = 8;
    pub const PROBE_INFO_OFFSET: u32 = 12;
    pub const DEBUG_VIEW_MODE_OFFSET: u32 = 16;
    pub const COUNT: u32 = 20;
    pub const SIZE: u32 = 80;
    pub const NAME: &str = "CCGlobal";
    pub const BINDING: u32 = PipelineGlobalBindings::UBO_GLOBAL as u32;
}

pub struct UBOCamera;
impl UBOCamera {
    pub const MAT_VIEW_OFFSET: u32 = 0;
    pub const MAT_VIEW_INV_OFFSET: u32 = 16;
    pub const MAT_PROJ_OFFSET: u32 = 32;
    pub const MAT_PROJ_INV_OFFSET: u32 = 48;
    pub const MAT_VIEW_PROJ_OFFSET: u32 = 64;
    pub const MAT_VIEW_PROJ_INV_OFFSET: u32 = 80;
    pub const CAMERA_POS_OFFSET: u32 = 96;
    pub const SURFACE_TRANSFORM_OFFSET: u32 = 100;
    pub const SCREEN_SCALE_OFFSET: u32 = 104;
    pub const EXPOSURE_OFFSET: u32 = 108;
    pub const MAIN_LIT_DIR_OFFSET: u32 = 112;
    pub const MAIN_LIT_COLOR_OFFSET: u32 = 116;
    pub const AMBIENT_SKY_OFFSET: u32 = 120;
    pub const AMBIENT_GROUND_OFFSET: u32 = 124;
    pub const GLOBAL_FOG_COLOR_OFFSET: u32 = 128;
    pub const GLOBAL_FOG_BASE_OFFSET: u32 = 132;
    pub const GLOBAL_FOG_ADD_OFFSET: u32 = 136;
    pub const NEAR_FAR_OFFSET: u32 = 140;
    pub const VIEW_PORT_OFFSET: u32 = 144;
    pub const COUNT: u32 = 148;
    pub const SIZE: u32 = 592;
    pub const NAME: &str = "CCCamera";
    pub const BINDING: u32 = PipelineGlobalBindings::UBO_CAMERA as u32;
}

pub struct UBOShadow;
impl UBOShadow {
    pub const MAT_LIGHT_VIEW_OFFSET: u32 = 0;
    pub const MAT_LIGHT_VIEW_PROJ_OFFSET: u32 = 16;
    pub const SHADOW_INV_PROJ_DEPTH_INFO_OFFSET: u32 = 32;
    pub const SHADOW_PROJ_DEPTH_INFO_OFFSET: u32 = 36;
    pub const SHADOW_PROJ_INFO_OFFSET: u32 = 40;
    pub const SHADOW_NEAR_FAR_LINEAR_SATURATION_INFO_OFFSET: u32 = 44;
    pub const SHADOW_WIDTH_HEIGHT_PCF_BIAS_INFO_OFFSET: u32 = 48;
    pub const SHADOW_LIGHT_PACKING_NBIAS_NULL_INFO_OFFSET: u32 = 52;
    pub const SHADOW_COLOR_OFFSET: u32 = 56;
    pub const PLANAR_NORMAL_DISTANCE_INFO_OFFSET: u32 = 60;
    pub const COUNT: u32 = 64;
    pub const SIZE: u32 = 256;
    pub const NAME: &str = "CCShadow";
    pub const BINDING: u32 = PipelineGlobalBindings::UBO_SHADOW as u32;
}

pub struct UBOCSM;
impl UBOCSM {
    pub const CSM_LEVEL_COUNT: u32 = 4;
    pub const CSM_VIEW_DIR_0_OFFSET: u32 = 0;
    pub const CSM_VIEW_DIR_1_OFFSET: u32 = 16;
    pub const CSM_VIEW_DIR_2_OFFSET: u32 = 32;
    pub const CSM_ATLAS_OFFSET: u32 = 48;
    pub const MAT_CSM_VIEW_PROJ_OFFSET: u32 = 64;
    pub const CSM_PROJ_DEPTH_INFO_OFFSET: u32 = 128;
    pub const CSM_PROJ_INFO_OFFSET: u32 = 144;
    pub const CSM_SPLITS_INFO_OFFSET: u32 = 160;
    pub const COUNT: u32 = 164;
    pub const SIZE: u32 = 656;
    pub const NAME: &str = "CCCSM";
    pub const BINDING: u32 = PipelineGlobalBindings::UBO_CSM as u32;
}

pub struct UBOLocal;
impl UBOLocal {
    pub const MAT_WORLD_OFFSET: u32 = 0;
    pub const MAT_WORLD_IT_OFFSET: u32 = 16;
    pub const LIGHTINGMAP_UVPARAM: u32 = 32;
    pub const LOCAL_SHADOW_BIAS: u32 = 36;
    pub const REFLECTION_PROBE_DATA1: u32 = 40;
    pub const REFLECTION_PROBE_DATA2: u32 = 44;
    pub const REFLECTION_PROBE_BLEND_DATA1: u32 = 48;
    pub const REFLECTION_PROBE_BLEND_DATA2: u32 = 52;
    pub const COUNT: u32 = 56;
    pub const SIZE: u32 = 224;
    pub const NAME: &str = "CCLocal";
    pub const BINDING: u32 = ModelLocalBindings::UBO_LOCAL as u32;
}

pub struct UBOForwardLight;
impl UBOForwardLight {
    pub const LIGHTS_PER_PASS: u32 = 1;
    pub const LIGHT_POS_OFFSET: u32 = 0;
    pub const LIGHT_COLOR_OFFSET: u32 = 4;
    pub const LIGHT_SIZE_RANGE_ANGLE_OFFSET: u32 = 8;
    pub const LIGHT_DIR_OFFSET: u32 = 12;
    pub const LIGHT_BOUNDING_SIZE_VS_OFFSET: u32 = 16;
    pub const COUNT: u32 = 20;
    pub const SIZE: u32 = 80;
    pub const NAME: &str = "CCForwardLight";
    pub const BINDING: u32 = ModelLocalBindings::UBO_FORWARD_LIGHTS as u32;
}

pub struct UBOMorph;
impl UBOMorph {
    pub const MAX_MORPH_TARGET_COUNT: u32 = 60;
    pub const OFFSET_OF_WEIGHTS: u32 = 0;
    pub const OFFSET_OF_DISPLACEMENT_TEXTURE_WIDTH: u32 = 240;
    pub const OFFSET_OF_DISPLACEMENT_TEXTURE_HEIGHT: u32 = 244;
    pub const OFFSET_OF_VERTICES_COUNT: u32 = 248;
    pub const COUNT_BASE_4_BYTES: u32 = 252;
    pub const SIZE: u32 = 1008;
    pub const NAME: &str = "CCMorph";
    pub const BINDING: u32 = ModelLocalBindings::UBO_MORPH as u32;
}

pub const UNIFORM_SHADOWMAP_NAME: &str = "cc_shadowMap";
pub const UNIFORM_SHADOWMAP_BINDING: u32 = PipelineGlobalBindings::SAMPLER_SHADOWMAP as u32;
pub const UNIFORM_ENVIRONMENT_NAME: &str = "cc_environment";
pub const UNIFORM_ENVIRONMENT_BINDING: u32 = PipelineGlobalBindings::SAMPLER_ENVIRONMENT as u32;
pub const UNIFORM_DIFFUSEMAP_NAME: &str = "cc_diffuseMap";
pub const UNIFORM_DIFFUSEMAP_BINDING: u32 = PipelineGlobalBindings::SAMPLER_DIFFUSEMAP as u32;
pub const UNIFORM_SPOT_SHADOW_MAP_NAME: &str = "cc_spotShadowMap";
pub const UNIFORM_SPOT_SHADOW_MAP_BINDING: u32 = PipelineGlobalBindings::SAMPLER_SPOT_SHADOW_MAP as u32;

pub const UNIFORM_LIGHTMAP_TEXTURE_NAME: &str = "cc_lightingMap";
pub const UNIFORM_LIGHTMAP_TEXTURE_BINDING: u32 = ModelLocalBindings::SAMPLER_LIGHTMAP as u32;
pub const UNIFORM_SPRITE_TEXTURE_NAME: &str = "cc_spriteTexture";
pub const UNIFORM_SPRITE_TEXTURE_BINDING: u32 = ModelLocalBindings::SAMPLER_SPRITE as u32;

pub const INST_MAT_WORLD: &str = "a_matWorld0";
pub const INST_SH: &str = "a_sh_linear_const_r";
pub const INST_JOINT_ANIM_INFO: &str = "a_jointAnimInfo";

pub const CAMERA_DEFAULT_MASK: u32 = 0xFFFFFFFF & !(0x2 | 0x4 | 0x8 | 0x10 | 0x20);
pub const MODEL_ALWAYS_MASK: u32 = 0xFFFFFFFF;

pub const MAX_BLOOM_FILTER_PASS_NUM: u32 = 6;
