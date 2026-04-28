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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CSMLevel {
    Level1 = 1,
    Level2 = 2,
    #[default]
    Level3 = 3,
    Level4 = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CSMOptimizationMode {
    None = 0,
    #[default]
    RemoveDuplicates = 1,
    DisableFarCascades = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraProjection {
    Ortho = 0,
    #[default]
    Perspective = 1,
    Unknown = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraFOVAxis {
    #[default]
    Vertical = 0,
    Horizontal = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraAperture {
    #[default]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraISO {
    #[default]
    Iso100 = 0,
    Iso200 = 1,
    Iso400 = 2,
    Iso800 = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraShutter {
    #[default]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraType {
    #[default]
    Default = -1,
    LeftEye = 0,
    RightEye = 1,
    Main = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrackingType {
    #[default]
    NoTracking = 0,
    PositionAndRotation = 1,
    Position = 2,
    Rotation = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraUsage {
    #[default]
    Game = 100,
    Editor = 0,
    GameView = 1,
    SceneView = 2,
    Preview = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModelType {
    #[default]
    Default = 0,
    BakedSkinning = 1,
    Skinning = 2,
    Batch = 3,
    Particle = 4,
    Line = 5,
}

pub const CAMERA_DEFAULT_MASK: u32 = 0xFFFFFFFF;

pub const SUN_ILLUM: f32 = 65000.0;
pub const SKY_ILLUM: f32 = 20000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UseReflectionProbeType {
    #[default]
    None = 0,
    BakedCubemap = 1,
    PlanarReflection = 2,
    BlendProbes = 3,
    BlendProbesAndSkybox = 4,
}
