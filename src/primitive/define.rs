/****************************************************************************
Rust port of Cocos Creator PrimitiveDefine (IGeometry)
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::math::Vec3;
use crate::renderer::gfx_base::shader::Attribute;
use crate::renderer::gfx_base::PrimitiveMode;

#[derive(Debug, Clone, Default)]
pub struct IGeometryOptions {
    pub include_normal: bool,
    pub include_uv: bool,
}

impl IGeometryOptions {
    pub fn new() -> Self {
        IGeometryOptions {
            include_normal: true,
            include_uv: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CustomAttribute {
    pub attr: Attribute,
    pub values: Vec<f32>,
}

#[derive(Debug, Clone, Default)]
pub struct IGeometry {
    pub positions: Vec<f32>,
    pub normals: Option<Vec<f32>>,
    pub uvs: Option<Vec<f32>>,
    pub tangents: Option<Vec<f32>>,
    pub colors: Option<Vec<f32>>,
    pub attributes: Option<Vec<Attribute>>,
    pub custom_attributes: Option<Vec<CustomAttribute>>,
    pub bounding_radius: Option<f32>,
    pub min_pos: Option<Vec3>,
    pub max_pos: Option<Vec3>,
    pub indices: Option<Vec<u32>>,
    pub primitive_mode: Option<PrimitiveMode>,
    pub double_sided: Option<bool>,
}

impl IGeometry {
    pub fn new() -> Self {
        IGeometry::default()
    }

    pub fn vertex_count(&self) -> usize {
        self.positions.len() / 3
    }
}
