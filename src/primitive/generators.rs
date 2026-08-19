/****************************************************************************
Rust port of Cocos Creator Primitive Geometry Generators
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::define::{IGeometry, IGeometryOptions};
use crate::math::Vec3;
use crate::renderer::gfx_base::PrimitiveMode;

const PI: f32 = std::f32::consts::PI;
const PI_2: f32 = 2.0 * PI;

pub fn box_geometry(options: Option<BoxOptions>) -> IGeometry {
    let opts = options.unwrap_or_default();
    let w = opts.width.unwrap_or(1.0);
    let h = opts.height.unwrap_or(1.0);
    let l = opts.length.unwrap_or(1.0);
    let ws = opts.width_segments.unwrap_or(1);
    let hs = opts.height_segments.unwrap_or(1);
    let ls = opts.length_segments.unwrap_or(1);

    let hw = w / 2.0;
    let hl = h / 2.0;
    let hle = l / 2.0;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let mut vertex_offset = 0u32;

    // Front face (z = +hle)
    generate_plane(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        vertex_offset,
        ws,
        hs,
        hw,
        hl,
        hle,
        Vec3::new(0.0, 0.0, 1.0),
        0.0,
        0.0,
    );
    vertex_offset += (ws + 1) * (hs + 1);

    // Back face (z = -hle)
    generate_plane(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        vertex_offset,
        ws,
        hs,
        hw,
        hl,
        hle,
        Vec3::new(0.0, 0.0, -1.0),
        PI_2,
        0.0,
    );
    vertex_offset += (ws + 1) * (hs + 1);

    // Left face (x = -hw)
    generate_plane(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        vertex_offset,
        ls,
        hs,
        hle,
        hl,
        hw,
        Vec3::new(-1.0, 0.0, 0.0),
        0.0,
        0.0,
    );
    vertex_offset += (ls + 1) * (hs + 1);

    // Right face (x = +hw)
    generate_plane(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        vertex_offset,
        ls,
        hs,
        hle,
        hl,
        hw,
        Vec3::new(1.0, 0.0, 0.0),
        PI_2,
        0.0,
    );
    vertex_offset += (ls + 1) * (hs + 1);

    // Top face (y = +hl)
    generate_plane(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        vertex_offset,
        ws,
        ls,
        hw,
        hle,
        hl,
        Vec3::new(0.0, 1.0, 0.0),
        0.0,
        0.0,
    );
    vertex_offset += (ws + 1) * (ls + 1);

    // Bottom face (y = -hl)
    generate_plane(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        vertex_offset,
        ws,
        ls,
        hw,
        hle,
        hl,
        Vec3::new(0.0, -1.0, 0.0),
        PI_2,
        0.0,
    );

    IGeometry {
        positions,
        normals: if opts.include_normal {
            Some(normals)
        } else {
            None
        },
        uvs: if opts.include_uv { Some(uvs) } else { None },
        indices: Some(indices),
        min_pos: Some(Vec3::new(-hw, -hl, -hle)),
        max_pos: Some(Vec3::new(hw, hl, hle)),
        bounding_radius: Some(hw.max(hl).max(hle)),
        primitive_mode: Some(PrimitiveMode::TriangleList),
        ..Default::default()
    }
}

#[allow(clippy::too_many_arguments)]
fn generate_plane(
    positions: &mut Vec<f32>,
    normals: &mut Vec<f32>,
    uvs: &mut Vec<f32>,
    indices: &mut Vec<u32>,
    offset: u32,
    seg_w: u32,
    seg_h: u32,
    half_w: f32,
    half_h: f32,
    depth: f32,
    normal: Vec3,
    _azimuth: f32,
    _polar: f32,
) {
    for iy in 0..=seg_h {
        for ix in 0..=seg_w {
            let u = ix as f32 / seg_w as f32 - 0.5;
            let v = iy as f32 / seg_h as f32 - 0.5;

            positions.push(u * 2.0 * half_w);
            positions.push(v * 2.0 * half_h);
            positions.push(depth);

            normals.push(normal.x);
            normals.push(normal.y);
            normals.push(normal.z);

            uvs.push(ix as f32 / seg_w as f32);
            uvs.push(1.0 - iy as f32 / seg_h as f32);
        }
    }

    for iy in 0..seg_h {
        for ix in 0..seg_w {
            let a = offset + ix + iy * (seg_w + 1);
            let b = offset + ix + (iy + 1) * (seg_w + 1);
            let c = offset + (ix + 1) + (iy + 1) * (seg_w + 1);
            let d = offset + (ix + 1) + iy * (seg_w + 1);

            indices.push(a);
            indices.push(b);
            indices.push(d);
            indices.push(b);
            indices.push(c);
            indices.push(d);
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoxOptions {
    pub include_normal: bool,
    pub include_uv: bool,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub length: Option<f32>,
    pub width_segments: Option<u32>,
    pub height_segments: Option<u32>,
    pub length_segments: Option<u32>,
}

impl Default for BoxOptions {
    fn default() -> Self {
        BoxOptions {
            include_normal: true,
            include_uv: true,
            width: None,
            height: None,
            length: None,
            width_segments: None,
            height_segments: None,
            length_segments: None,
        }
    }
}

pub fn sphere(radius: f32, options: Option<SphereOptions>) -> IGeometry {
    let opts = options.unwrap_or_default();
    let segments = opts.segments.unwrap_or(32);

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for lat in 0..=segments {
        let theta = lat as f32 * PI / segments as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for lon in 0..=segments {
            let phi = lon as f32 * PI_2 / segments as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = cos_phi * sin_theta;
            let y = cos_theta;
            let z = sin_phi * sin_theta;

            positions.push(x * radius);
            positions.push(y * radius);
            positions.push(z * radius);

            if opts.include_normal {
                normals.push(x);
                normals.push(y);
                normals.push(z);
            }

            if opts.include_uv {
                uvs.push(lon as f32 / segments as f32);
                uvs.push(lat as f32 / segments as f32);
            }
        }
    }

    for lat in 0..segments {
        for lon in 0..segments {
            let first = lat * (segments + 1) + lon;
            let second = first + segments + 1;

            indices.push(first);
            indices.push(second);
            indices.push(first + 1);
            indices.push(second);
            indices.push(second + 1);
            indices.push(first + 1);
        }
    }

    IGeometry {
        positions,
        normals: if opts.include_normal {
            Some(normals)
        } else {
            None
        },
        uvs: if opts.include_uv { Some(uvs) } else { None },
        indices: Some(indices),
        min_pos: Some(Vec3::new(-radius, -radius, -radius)),
        max_pos: Some(Vec3::new(radius, radius, radius)),
        bounding_radius: Some(radius),
        primitive_mode: Some(PrimitiveMode::TriangleList),
        ..Default::default()
    }
}

#[derive(Debug, Clone)]
pub struct SphereOptions {
    pub include_normal: bool,
    pub include_uv: bool,
    pub segments: Option<u32>,
}

impl Default for SphereOptions {
    fn default() -> Self {
        SphereOptions {
            include_normal: true,
            include_uv: true,
            segments: None,
        }
    }
}

pub fn cylinder(
    radius_top: f32,
    radius_bottom: f32,
    height: f32,
    options: Option<CylinderOptions>,
) -> IGeometry {
    let opts = options.unwrap_or_default();
    let radial_segments = opts.radial_segments;
    let height_segments = opts.height_segments;
    let capped = opts.capped;
    let arc = opts.arc;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let half_height = height / 2.0;
    let slope = (radius_bottom - radius_top) / height;
    let mut vertex_offset = 0u32;

    for iy in 0..=height_segments {
        let y = half_height - iy as f32 * height / height_segments as f32;
        let radius = y * slope + radius_top;

        for ix in 0..=radial_segments {
            let u = ix as f32 / radial_segments as f32;
            let phi = u * arc;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            positions.push(radius * sin_phi);
            positions.push(y);
            positions.push(radius * cos_phi);

            let normal_len = (1.0 + slope * slope).sqrt();
            normals.push(sin_phi / normal_len);
            normals.push(slope / normal_len);
            normals.push(cos_phi / normal_len);

            uvs.push(u);
            uvs.push(iy as f32 / height_segments as f32);
        }
    }

    for iy in 0..height_segments {
        for ix in 0..radial_segments {
            let a = vertex_offset + ix + iy * (radial_segments + 1);
            let b = vertex_offset + ix + (iy + 1) * (radial_segments + 1);
            let c = vertex_offset + (ix + 1) + (iy + 1) * (radial_segments + 1);
            let d = vertex_offset + (ix + 1) + iy * (radial_segments + 1);

            indices.push(a);
            indices.push(b);
            indices.push(d);
            indices.push(b);
            indices.push(c);
            indices.push(d);
        }
    }
    vertex_offset += (height_segments + 1) * (radial_segments + 1);

    if capped && radius_top > 0.0 {
        // Top cap
        for ix in 0..=radial_segments {
            let u = ix as f32 / radial_segments as f32;
            let phi = u * arc;
            positions.push(0.0);
            positions.push(half_height);
            positions.push(0.0);
            normals.push(0.0);
            normals.push(1.0);
            normals.push(0.0);
            uvs.push(u);
            uvs.push(1.0);

            if ix > 0 {
                positions.push(radius_top * phi.sin());
                positions.push(half_height);
                positions.push(radius_top * phi.cos());
                normals.push(0.0);
                normals.push(1.0);
                normals.push(0.0);
                uvs.push(u);
                uvs.push(1.0);

                indices.push(vertex_offset);
                indices.push(vertex_offset + ix * 2);
                indices.push(vertex_offset + ix * 2 - 2);
            }
        }
        vertex_offset += (radial_segments + 1) * 2;
    }

    if capped && radius_bottom > 0.0 {
        // Bottom cap
        for ix in 0..=radial_segments {
            let u = ix as f32 / radial_segments as f32;
            let phi = u * arc;
            positions.push(0.0);
            positions.push(-half_height);
            positions.push(0.0);
            normals.push(0.0);
            normals.push(-1.0);
            normals.push(0.0);
            uvs.push(u);
            uvs.push(0.0);

            if ix > 0 {
                positions.push(radius_bottom * phi.sin());
                positions.push(-half_height);
                positions.push(radius_bottom * phi.cos());
                normals.push(0.0);
                normals.push(-1.0);
                normals.push(0.0);
                uvs.push(u);
                uvs.push(0.0);

                indices.push(vertex_offset);
                indices.push(vertex_offset + ix * 2 - 2);
                indices.push(vertex_offset + ix * 2);
            }
        }
    }

    IGeometry {
        positions,
        normals: Some(normals),
        uvs: Some(uvs),
        indices: Some(indices),
        min_pos: Some(Vec3::new(
            -radius_bottom.max(radius_top),
            -half_height,
            -radius_bottom.max(radius_top),
        )),
        max_pos: Some(Vec3::new(
            radius_bottom.max(radius_top),
            half_height,
            radius_bottom.max(radius_top),
        )),
        bounding_radius: Some(radius_bottom.max(radius_top).max(half_height)),
        primitive_mode: Some(PrimitiveMode::TriangleList),
        ..Default::default()
    }
}

#[derive(Debug, Clone)]
pub struct CylinderOptions {
    pub include_normal: bool,
    pub include_uv: bool,
    pub radial_segments: u32,
    pub height_segments: u32,
    pub capped: bool,
    pub arc: f32,
}

impl Default for CylinderOptions {
    fn default() -> Self {
        CylinderOptions {
            include_normal: true,
            include_uv: true,
            radial_segments: 32,
            height_segments: 1,
            capped: true,
            arc: PI_2,
        }
    }
}

pub fn cone(radius: f32, height: f32, options: Option<CylinderOptions>) -> IGeometry {
    cylinder(0.0, radius, height, options)
}

pub fn quad(options: Option<IGeometryOptions>) -> IGeometry {
    let opts = options.unwrap_or_default();

    let positions = vec![
        -0.5, -0.5, 0.0, -0.5, 0.5, 0.0, 0.5, 0.5, 0.0, 0.5, -0.5, 0.0,
    ];

    let normals = if opts.include_normal {
        Some(vec![
            0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0,
        ])
    } else {
        None
    };

    let uvs = if opts.include_uv {
        Some(vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0])
    } else {
        None
    };

    let indices = Some(vec![0, 1, 2, 0, 2, 3]);

    IGeometry {
        positions,
        normals,
        uvs,
        indices,
        min_pos: Some(Vec3::new(-0.5, -0.5, 0.0)),
        max_pos: Some(Vec3::new(0.5, 0.5, 0.0)),
        bounding_radius: Some(0.5 * 2.0_f32.sqrt()),
        primitive_mode: Some(PrimitiveMode::TriangleList),
        ..Default::default()
    }
}

pub fn plane(options: Option<PlaneOptions>) -> IGeometry {
    let opts = options.unwrap_or_default();
    let width = opts.width;
    let length = opts.length;
    let ws = opts.width_segments;
    let ls = opts.length_segments;

    let hw = width / 2.0;
    let hl = length / 2.0;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for iy in 0..=ls {
        for ix in 0..=ws {
            let x = ix as f32 / ws as f32 * width - hw;
            let z = iy as f32 / ls as f32 * length - hl;

            positions.push(x);
            positions.push(0.0);
            positions.push(z);

            normals.push(0.0);
            normals.push(1.0);
            normals.push(0.0);

            uvs.push(ix as f32 / ws as f32);
            uvs.push(iy as f32 / ls as f32);
        }
    }

    for iy in 0..ls {
        for ix in 0..ws {
            let a = ix + iy * (ws + 1);
            let b = ix + (iy + 1) * (ws + 1);
            let c = (ix + 1) + (iy + 1) * (ws + 1);
            let d = (ix + 1) + iy * (ws + 1);

            indices.push(a);
            indices.push(b);
            indices.push(d);
            indices.push(b);
            indices.push(c);
            indices.push(d);
        }
    }

    IGeometry {
        positions,
        normals: if opts.include_normal {
            Some(normals)
        } else {
            None
        },
        uvs: if opts.include_uv { Some(uvs) } else { None },
        indices: Some(indices),
        min_pos: Some(Vec3::new(-hw, 0.0, -hl)),
        max_pos: Some(Vec3::new(hw, 0.0, hl)),
        bounding_radius: Some(hw.max(hl)),
        primitive_mode: Some(PrimitiveMode::TriangleList),
        ..Default::default()
    }
}

#[derive(Debug, Clone)]
pub struct PlaneOptions {
    pub include_normal: bool,
    pub include_uv: bool,
    pub width: f32,
    pub length: f32,
    pub width_segments: u32,
    pub length_segments: u32,
}

impl Default for PlaneOptions {
    fn default() -> PlaneOptions {
        PlaneOptions {
            include_normal: true,
            include_uv: true,
            width: 10.0,
            length: 10.0,
            width_segments: 10,
            length_segments: 10,
        }
    }
}

pub fn transform_translate(geometry: &mut IGeometry, offset: Vec3) {
    for i in (0..geometry.positions.len()).step_by(3) {
        geometry.positions[i] += offset.x;
        geometry.positions[i + 1] += offset.y;
        geometry.positions[i + 2] += offset.z;
    }
    if let Some(min) = geometry.min_pos.as_mut() {
        *min += offset;
    }
    if let Some(max) = geometry.max_pos.as_mut() {
        *max += offset;
    }
}

pub fn transform_scale(geometry: &mut IGeometry, scale: Vec3) {
    for i in (0..geometry.positions.len()).step_by(3) {
        geometry.positions[i] *= scale.x;
        geometry.positions[i + 1] *= scale.y;
        geometry.positions[i + 2] *= scale.z;
    }
    if let Some(min) = geometry.min_pos.as_mut() {
        *min = Vec3::new(min.x * scale.x, min.y * scale.y, min.z * scale.z);
    }
    if let Some(max) = geometry.max_pos.as_mut() {
        *max = Vec3::new(max.x * scale.x, max.y * scale.y, max.z * scale.z);
    }
    if let Some(r) = geometry.bounding_radius.as_mut() {
        *r *= scale.x.max(scale.y).max(scale.z);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec3;

    #[test]
    fn test_box_default() {
        let geo = box_geometry(None);
        assert!(!geo.positions.is_empty());
        assert!(geo.indices.is_some());
        assert!(geo.normals.is_some());
        assert!(geo.uvs.is_some());
        let min = geo.min_pos.unwrap();
        let max = geo.max_pos.unwrap();
        assert_eq!(min.x, -0.5);
        assert_eq!(max.x, 0.5);
    }

    #[test]
    fn test_box_no_normal_no_uv() {
        let geo = box_geometry(Some(BoxOptions {
            include_normal: false,
            include_uv: false,
            ..Default::default()
        }));
        assert!(geo.normals.is_none());
        assert!(geo.uvs.is_none());
    }

    #[test]
    fn test_sphere_default() {
        let geo = sphere(1.0, None);
        assert!(!geo.positions.is_empty());
        assert!(geo.indices.is_some());
        let min = geo.min_pos.unwrap();
        assert_eq!(min.x, -1.0);
        assert_eq!(min.y, -1.0);
        assert_eq!(min.z, -1.0);
    }

    #[test]
    fn test_sphere_small() {
        let geo = sphere(
            0.5,
            Some(SphereOptions {
                segments: Some(8),
                include_normal: true,
                include_uv: true,
            }),
        );
        let vc = geo.vertex_count();
        assert_eq!(vc, 81); // (8+1) * (8+1) = 81
    }

    #[test]
    fn test_cylinder_default() {
        let geo = cylinder(0.5, 0.5, 2.0, None);
        assert!(!geo.positions.is_empty());
        assert!(geo.indices.is_some());
    }

    #[test]
    fn test_cone_default() {
        let geo = cone(0.5, 1.0, None);
        assert!(!geo.positions.is_empty());
        let min = geo.min_pos.unwrap();
        let max = geo.max_pos.unwrap();
        assert!(min.x < 0.0);
        assert!(max.y > 0.0);
    }

    #[test]
    fn test_quad() {
        let geo = quad(None);
        assert_eq!(geo.positions.len(), 12); // 4 vertices * 3 components
        assert!(geo.indices.is_some());
        let indices = geo.indices.unwrap();
        assert_eq!(indices.len(), 6); // 2 triangles
    }

    #[test]
    fn test_plane_default() {
        let geo = plane(None);
        assert!(!geo.positions.is_empty());
        let min = geo.min_pos.unwrap();
        assert_eq!(min.x, -5.0);
        assert_eq!(min.z, -5.0);
    }

    #[test]
    fn test_translate() {
        let mut geo = box_geometry(None);
        transform_translate(&mut geo, Vec3::new(10.0, 0.0, 0.0));
        let min = geo.min_pos.unwrap();
        assert_eq!(min.x, 9.5);
    }

    #[test]
    fn test_scale() {
        let mut geo = box_geometry(None);
        transform_scale(&mut geo, Vec3::new(2.0, 2.0, 2.0));
        let max = geo.max_pos.unwrap();
        assert_eq!(max.x, 1.0);
    }

    #[test]
    fn test_igeometry_vertex_count() {
        let geo = quad(None);
        assert_eq!(geo.vertex_count(), 4);
    }
}
