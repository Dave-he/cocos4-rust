/****************************************************************************
Rust port of Cocos Creator Pipeline Define System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::math::Vec3;
use crate::math::Vec4;
use crate::math::Color;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CSMLevel {
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
    Level4 = 4,
}

impl Default for CSMLevel {
    fn default() -> Self {
        CSMLevel::Level2
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraProjection {
    Perspective = 0,
    Ortho = 1,
}

impl Default for CameraProjection {
    fn default() -> Self {
        CameraProjection::Perspective
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraFOV {
    Vertical = 0,
    Horizontal = 1,
}

impl Default for CameraFOV {
    fn default() -> Self {
        CameraFOV::Vertical
    }
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
