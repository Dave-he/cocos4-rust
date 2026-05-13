/****************************************************************************
Rust port of Cocos Creator Primitive Module
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

pub mod define;
pub mod generators;

pub use define::{IGeometry, IGeometryOptions, CustomAttribute};
pub use generators::{
    box_geometry, BoxOptions,
    sphere, SphereOptions,
    cylinder, CylinderOptions,
    cone,
    quad,
    plane, PlaneOptions,
    transform_translate, transform_scale,
};
