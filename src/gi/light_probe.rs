/****************************************************************************
Rust port of Cocos Creator Light Probe
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/
// SPDX-License-Identifier: MIT

use crate::math::{Vec3, Vec4};

const EPSILON: f32 = 1e-6;
use crate::gi::delaunay::{Delaunay, Tetrahedron, Vertex};
use crate::gi::polynomial_solver::PolynomialSolver;
use crate::gi::sh::SH_BASIS_COUNT;

#[derive(Debug, Clone)]
pub struct LightProbesData {
    probes: Vec<Vertex>,
    tetrahedrons: Vec<Tetrahedron>,
}

impl Default for LightProbesData {
    fn default() -> Self {
        Self::new()
    }
}

impl LightProbesData {
    pub fn new() -> Self {
        Self {
            probes: Vec::new(),
            tetrahedrons: Vec::new(),
        }
    }

    pub fn probes(&self) -> &Vec<Vertex> {
        &self.probes
    }
    pub fn tetrahedrons(&self) -> &Vec<Tetrahedron> {
        &self.tetrahedrons
    }

    pub fn empty(&self) -> bool {
        self.probes.is_empty() || self.tetrahedrons.is_empty()
    }

    pub fn reset(&mut self) {
        self.probes.clear();
        self.tetrahedrons.clear();
    }

    pub fn update_probes(&mut self, points: &[Vec3]) {
        self.probes
            .resize_with(points.len(), || Vertex::new(Vec3::ZERO));
        for i in 0..points.len() {
            self.probes[i].position = points[i];
        }
    }

    pub fn update_tetrahedrons(&mut self) {
        let mut delaunay = Delaunay::new(self.probes.clone());
        self.tetrahedrons = delaunay.build();
    }

    pub fn has_coefficients(&self) -> bool {
        !self.empty() && !self.probes[0].coefficients.is_empty()
    }

    pub fn get_interpolation_sh_coefficients(
        &self,
        tet_index: i32,
        weights: &Vec4,
        coefficients: &mut Vec<Vec3>,
    ) -> bool {
        if !self.has_coefficients() {
            return false;
        }
        let tet_idx = tet_index as usize;
        if tet_idx >= self.tetrahedrons.len() {
            return false;
        }
        let tetrahedron = &self.tetrahedrons[tet_idx];
        let c0 = &self.probes[tetrahedron.vertex0 as usize].coefficients;
        let c1 = &self.probes[tetrahedron.vertex1 as usize].coefficients;
        let c2 = &self.probes[tetrahedron.vertex2 as usize].coefficients;
        coefficients.resize(SH_BASIS_COUNT, Vec3::ZERO);
        if tetrahedron.vertex3 >= 0 {
            let c3 = &self.probes[tetrahedron.vertex3 as usize].coefficients;
            for i in 0..SH_BASIS_COUNT {
                coefficients[i] =
                    c0[i] * weights.x + c1[i] * weights.y + c2[i] * weights.z + c3[i] * weights.w;
            }
        } else {
            for i in 0..SH_BASIS_COUNT {
                coefficients[i] = c0[i] * weights.x + c1[i] * weights.y + c2[i] * weights.z;
            }
        }
        true
    }

    pub fn get_interpolation_weights(&self, position: &Vec3, tet_index: i32) -> (i32, Vec4) {
        let tetrahedron_count = self.tetrahedrons.len() as i32;
        let mut tet_index = if tet_index < 0 || tet_index >= tetrahedron_count {
            0
        } else {
            tet_index
        };
        let mut weights = Vec4::new(0.0, 0.0, 0.0, 0.0);
        let mut last_index = -1;
        for _ in 0..tetrahedron_count {
            let tetrahedron = &self.tetrahedrons[tet_index as usize];
            self.get_barycentric_coord(position, tetrahedron, &mut weights);
            if weights.x >= 0.0 && weights.y >= 0.0 && weights.z >= 0.0 && weights.w >= 0.0 {
                break;
            }
            let next_index =
                if weights.x < weights.y && weights.x < weights.z && weights.x < weights.w {
                    tetrahedron.neighbours[0]
                } else if weights.y < weights.z && weights.y < weights.w {
                    tetrahedron.neighbours[1]
                } else if weights.z < weights.w {
                    tetrahedron.neighbours[2]
                } else {
                    tetrahedron.neighbours[3]
                };
            if last_index == next_index {
                break;
            }
            last_index = tet_index;
            tet_index = next_index;
        }
        (tet_index, weights)
    }

    fn get_triangle_barycentric_coord(p0: &Vec3, p1: &Vec3, p2: &Vec3, position: &Vec3) -> Vec3 {
        let v1 = *p1 - *p0;
        let v2 = *p2 - *p0;
        let normal = Vec3::cross_vecs(&v1, &v2);
        if normal.length_squared() <= EPSILON {
            return Vec3::ZERO;
        }
        let n_norm = normal.get_normalized();
        let area012_inv = 1.0 / Vec3::dot_vecs(&n_norm, &normal);
        let edge_p0 = *p0 - *position;
        let edge_p1 = *p1 - *position;
        let edge_p2 = *p2 - *position;
        let cross_p12 = Vec3::cross_vecs(&edge_p1, &edge_p2);
        let alpha = Vec3::dot_vecs(&n_norm, &cross_p12) * area012_inv;
        let cross_p20 = Vec3::cross_vecs(&edge_p2, &edge_p0);
        let beta = Vec3::dot_vecs(&n_norm, &cross_p20) * area012_inv;
        Vec3::new(alpha, beta, 1.0 - alpha - beta)
    }

    fn get_barycentric_coord(
        &self,
        position: &Vec3,
        tetrahedron: &Tetrahedron,
        weights: &mut Vec4,
    ) {
        if tetrahedron.vertex3 >= 0 {
            self.get_tetrahedron_barycentric_coord(position, tetrahedron, weights);
        } else {
            self.get_outer_cell_barycentric_coord(position, tetrahedron, weights);
        }
    }

    fn get_tetrahedron_barycentric_coord(
        &self,
        position: &Vec3,
        tetrahedron: &Tetrahedron,
        weights: &mut Vec4,
    ) {
        let p3 = self.probes[tetrahedron.vertex3 as usize].position;
        let result = *position - p3;
        let transformed = tetrahedron.matrix.transform_vec3(&result);
        weights.x = transformed.x;
        weights.y = transformed.y;
        weights.z = transformed.z;
        weights.w = 1.0 - transformed.x - transformed.y - transformed.z;
    }

    fn get_outer_cell_barycentric_coord(
        &self,
        position: &Vec3,
        tetrahedron: &Tetrahedron,
        weights: &mut Vec4,
    ) {
        let p0 = self.probes[tetrahedron.vertex0 as usize].position;
        let p1 = self.probes[tetrahedron.vertex1 as usize].position;
        let p2 = self.probes[tetrahedron.vertex2 as usize].position;
        let n0 = self.probes[tetrahedron.vertex0 as usize].normal;
        let n1 = self.probes[tetrahedron.vertex1 as usize].normal;
        let n2 = self.probes[tetrahedron.vertex2 as usize].normal;
        let edge1 = p1 - p0;
        let edge2 = p2 - p0;
        let normal2 = Vec3::cross_vecs(&edge1, &edge2);
        let v = *position - p0;
        let t_dot = Vec3::dot_vecs(&v, &normal2);
        if t_dot < 0.0 {
            weights.x = 0.0;
            weights.y = 0.0;
            weights.z = 0.0;
            weights.w = -1.0;
            return;
        }
        let coefficients = tetrahedron.matrix.transform_vec3(position) + tetrahedron.offset;
        let t = if tetrahedron.vertex3 == -1 {
            PolynomialSolver::get_cubic_unique_root(coefficients.x, coefficients.y, coefficients.z)
        } else {
            PolynomialSolver::get_quadratic_unique_root(
                coefficients.x,
                coefficients.y,
                coefficients.z,
            )
        };
        let vp0 = p0 + n0 * t;
        let vp1 = p1 + n1 * t;
        let vp2 = p2 + n2 * t;
        let result = Self::get_triangle_barycentric_coord(&vp0, &vp1, &vp2, position);
        weights.x = result.x;
        weights.y = result.y;
        weights.z = result.z;
        weights.w = 0.0;
    }
}

#[derive(Debug, Clone)]
pub struct LightProbes {
    gi_scale: f32,
    gi_samples: u32,
    bounces: u32,
    reduce_ringing: f32,
    show_probe: bool,
    show_wireframe: bool,
    show_convex: bool,
    data: Option<LightProbesData>,
    light_probe_sphere_volume: f32,
}

impl Default for LightProbes {
    fn default() -> Self {
        Self::new()
    }
}

impl LightProbes {
    pub fn new() -> Self {
        Self {
            gi_scale: 1.0,
            gi_samples: 1024,
            bounces: 2,
            reduce_ringing: 0.0,
            show_probe: true,
            show_wireframe: true,
            show_convex: false,
            data: None,
            light_probe_sphere_volume: 1.0,
        }
    }

    pub fn initialize(&mut self, info: &LightProbeInfo) {
        self.gi_scale = info.gi_scale;
        self.gi_samples = info.gi_samples;
        self.bounces = info.bounces;
        self.reduce_ringing = info.reduce_ringing;
        self.show_probe = info.show_probe;
        self.show_wireframe = info.show_wireframe;
        self.show_convex = info.show_convex;
        self.light_probe_sphere_volume = info.light_probe_sphere_volume;
        self.data = Some(info.data.clone());
    }

    pub fn gi_scale(&self) -> f32 {
        self.gi_scale
    }
    pub fn set_gi_scale(&mut self, val: f32) {
        self.gi_scale = val;
    }
    pub fn gi_samples(&self) -> u32 {
        self.gi_samples
    }
    pub fn set_gi_samples(&mut self, val: u32) {
        self.gi_samples = val;
    }
    pub fn bounces(&self) -> u32 {
        self.bounces
    }
    pub fn set_bounces(&mut self, val: u32) {
        self.bounces = val;
    }
    pub fn reduce_ringing(&self) -> f32 {
        self.reduce_ringing
    }
    pub fn set_reduce_ringing(&mut self, val: f32) {
        self.reduce_ringing = val;
    }
    pub fn show_probe(&self) -> bool {
        self.show_probe
    }
    pub fn set_show_probe(&mut self, val: bool) {
        self.show_probe = val;
    }
    pub fn show_wireframe(&self) -> bool {
        self.show_wireframe
    }
    pub fn set_show_wireframe(&mut self, val: bool) {
        self.show_wireframe = val;
    }
    pub fn show_convex(&self) -> bool {
        self.show_convex
    }
    pub fn set_show_convex(&mut self, val: bool) {
        self.show_convex = val;
    }
    pub fn data(&self) -> &Option<LightProbesData> {
        &self.data
    }
    pub fn set_data(&mut self, val: LightProbesData) {
        self.data = Some(val);
    }
    pub fn light_probe_sphere_volume(&self) -> f32 {
        self.light_probe_sphere_volume
    }
    pub fn set_light_probe_sphere_volume(&mut self, val: f32) {
        self.light_probe_sphere_volume = val;
    }

    pub fn empty(&self) -> bool {
        match &self.data {
            Some(d) => d.empty(),
            None => true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LightProbeInfo {
    pub gi_scale: f32,
    pub light_probe_sphere_volume: f32,
    pub gi_samples: u32,
    pub bounces: u32,
    pub reduce_ringing: f32,
    pub show_probe: bool,
    pub show_wireframe: bool,
    pub show_convex: bool,
    pub data: LightProbesData,
}

impl Default for LightProbeInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl LightProbeInfo {
    pub fn new() -> Self {
        Self {
            gi_scale: 1.0,
            light_probe_sphere_volume: 1.0,
            gi_samples: 1024,
            bounces: 2,
            reduce_ringing: 0.0,
            show_probe: true,
            show_wireframe: true,
            show_convex: false,
            data: LightProbesData::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_probes_data_empty() {
        let data = LightProbesData::new();
        assert!(data.empty());
    }

    #[test]
    fn test_light_probes_data_update_probes() {
        let mut data = LightProbesData::new();
        let points = vec![Vec3::ZERO, Vec3::UNIT_X, Vec3::UNIT_Y, Vec3::UNIT_Z];
        data.update_probes(&points);
        assert_eq!(data.probes().len(), 4);
        assert!(data.empty() || data.tetrahedrons().is_empty());
    }

    #[test]
    fn test_light_probes_data_reset() {
        let mut data = LightProbesData::new();
        data.update_probes(&[Vec3::ZERO, Vec3::UNIT_X]);
        data.reset();
        assert!(data.empty());
    }

    #[test]
    fn test_light_probes_default() {
        let lp = LightProbes::new();
        assert_eq!(lp.gi_scale(), 1.0);
        assert_eq!(lp.gi_samples(), 1024);
        assert_eq!(lp.bounces(), 2);
        assert!(lp.show_probe());
        assert!(lp.show_wireframe());
        assert!(!lp.show_convex());
        assert!(lp.empty());
    }

    #[test]
    fn test_light_probes_initialize() {
        let info = LightProbeInfo::new();
        let mut lp = LightProbes::new();
        lp.initialize(&info);
        assert_eq!(lp.gi_scale(), 1.0);
        assert_eq!(lp.gi_samples(), 1024);
    }

    #[test]
    fn test_light_probe_info_default() {
        let info = LightProbeInfo::default();
        assert_eq!(info.gi_scale, 1.0);
        assert_eq!(info.gi_samples, 1024);
        assert!(info.show_probe);
    }

    #[test]
    fn test_light_probes_data_has_coefficients_empty() {
        let data = LightProbesData::new();
        assert!(!data.has_coefficients());
    }

    #[test]
    fn test_light_probes_setters() {
        let mut lp = LightProbes::new();
        lp.set_gi_scale(2.0);
        assert_eq!(lp.gi_scale(), 2.0);
        lp.set_bounces(4);
        assert_eq!(lp.bounces(), 4);
        lp.set_show_convex(true);
        assert!(lp.show_convex());
    }
}
