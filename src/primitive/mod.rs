/****************************************************************************
Rust port of Cocos Creator Primitive Module
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

pub mod define;
pub mod generators;

pub use define::{CustomAttribute, IGeometry, IGeometryOptions};
pub use generators::{
    box_geometry, cone, cylinder, plane, quad, sphere, transform_scale, transform_translate,
    BoxOptions, CylinderOptions, PlaneOptions, SphereOptions,
};
