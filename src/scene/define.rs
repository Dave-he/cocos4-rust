#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightType {
    Directional = 0,
    Sphere = 1,
    Spot = 2,
    Point = 3,
    RangedDirectional = 4,
    Unknown = 5,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CSMLevel {
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
    Level4 = 4,
}

impl Default for CSMLevel {
    fn default() -> Self {
        CSMLevel::Level3
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CSMOptimizationMode {
    None = 0,
    RemoveDuplicates = 1,
    DisableFarCascades = 2,
}

impl Default for CSMOptimizationMode {
    fn default() -> Self {
        CSMOptimizationMode::RemoveDuplicates
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraProjection {
    Ortho = 0,
    Perspective = 1,
    Unknown = 2,
}

impl Default for CameraProjection {
    fn default() -> Self {
        CameraProjection::Perspective
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraFOVAxis {
    Vertical = 0,
    Horizontal = 1,
}

impl Default for CameraFOVAxis {
    fn default() -> Self {
        CameraFOVAxis::Vertical
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraAperture {
    F1_8 = 0,
    F2_0,
    F2_2,
    F2_5,
    F2_8,
    F3_2,
    F3_5,
    F4_0,
    F4_5,
    F5_0,
    F5_6,
    F6_3,
    F7_1,
    F8_0,
    F9_0,
    F10_0,
    F11_0,
    F13_0,
    F14_0,
    F16_0,
    F18_0,
    F20_0,
    F22_0,
}

impl Default for CameraAperture {
    fn default() -> Self {
        CameraAperture::F16_0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraISO {
    Iso100 = 0,
    Iso200 = 1,
    Iso400 = 2,
    Iso800 = 3,
}

impl Default for CameraISO {
    fn default() -> Self {
        CameraISO::Iso100
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraShutter {
    D1 = 0,
    D2,
    D4,
    D8,
    D15,
    D30,
    D60,
    D125,
    D250,
    D500,
    D1000,
    D2000,
    D4000,
}

impl Default for CameraShutter {
    fn default() -> Self {
        CameraShutter::D125
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraType {
    Default = -1,
    LeftEye = 0,
    RightEye = 1,
    Main = 2,
}

impl Default for CameraType {
    fn default() -> Self {
        CameraType::Default
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackingType {
    NoTracking = 0,
    PositionAndRotation = 1,
    Position = 2,
    Rotation = 3,
}

impl Default for TrackingType {
    fn default() -> Self {
        TrackingType::NoTracking
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraUsage {
    Editor = 0,
    GameView = 1,
    SceneView = 2,
    Preview = 3,
    Game = 100,
}

impl Default for CameraUsage {
    fn default() -> Self {
        CameraUsage::Game
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelType {
    Default = 0,
    BakedSkinning = 1,
    Skinning = 2,
    Batch = 3,
    Particle = 4,
    Line = 5,
}

impl Default for ModelType {
    fn default() -> Self {
        ModelType::Default
    }
}

pub const CAMERA_DEFAULT_MASK: u32 = 0xFFFFFFFF;

pub const SUN_ILLUM: f32 = 65000.0;
pub const SKY_ILLUM: f32 = 20000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseReflectionProbeType {
    None = 0,
    BakedCubemap = 1,
    PlanarReflection = 2,
    BlendProbes = 3,
    BlendProbesAndSkybox = 4,
}

impl Default for UseReflectionProbeType {
    fn default() -> Self {
        UseReflectionProbeType::None
    }
}
