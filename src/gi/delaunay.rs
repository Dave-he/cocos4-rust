/****************************************************************************
Rust port of Cocos Creator Delaunay triangulation
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/
// SPDX-License-Identifier: MIT

use crate::gi::sh::SH_BASIS_COUNT;
use crate::math::{Mat3, Vec3};

const EPSILON: f32 = 1e-6;

#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub coefficients: Vec<Vec3>,
}

impl Vertex {
    pub fn new(pos: Vec3) -> Self {
        Self {
            position: pos,
            normal: Vec3::new(0.0, 0.0, 0.0),
            coefficients: vec![Vec3::ZERO; SH_BASIS_COUNT],
        }
    }
}

#[derive(Debug, Clone)]
struct Edge {
    tetrahedron: i32,
    index: i32,
    vertex0: i32,
    vertex1: i32,
}

impl Edge {
    fn new(tet: i32, i: i32, v0: i32, v1: i32) -> Self {
        let (sv0, sv1) = if v0 < v1 { (v0, v1) } else { (v1, v0) };
        Self {
            tetrahedron: tet,
            index: i,
            vertex0: sv0,
            vertex1: sv1,
        }
    }

    #[allow(dead_code)]
    fn set(&mut self, tet: i32, i: i32, v0: i32, v1: i32) {
        self.tetrahedron = tet;
        self.index = i;
        let (sv0, sv1) = if v0 < v1 { (v0, v1) } else { (v1, v0) };
        self.vertex0 = sv0;
        self.vertex1 = sv1;
    }

    fn is_same(&self, other: &Edge) -> bool {
        self.vertex0 == other.vertex0 && self.vertex1 == other.vertex1
    }
}

#[derive(Debug, Clone)]
struct Triangle {
    invalid: bool,
    is_outer_face: bool,
    tetrahedron: i32,
    index: i32,
    vertex0: i32,
    vertex1: i32,
    vertex2: i32,
    vertex3: i32,
}

impl Triangle {
    fn new(tet: i32, i: i32, v0: i32, v1: i32, v2: i32, v3: i32) -> Self {
        let (sv0, sv1, sv2) = sort_three(v0, v1, v2);
        Self {
            invalid: false,
            is_outer_face: true,
            tetrahedron: tet,
            index: i,
            vertex0: sv0,
            vertex1: sv1,
            vertex2: sv2,
            vertex3: v3,
        }
    }

    fn set(&mut self, tet: i32, i: i32, v0: i32, v1: i32, v2: i32, v3: i32) {
        let (sv0, sv1, sv2) = sort_three(v0, v1, v2);
        self.invalid = false;
        self.is_outer_face = true;
        self.tetrahedron = tet;
        self.index = i;
        self.vertex0 = sv0;
        self.vertex1 = sv1;
        self.vertex2 = sv2;
        self.vertex3 = v3;
    }

    fn is_same(&self, other: &Triangle) -> bool {
        self.vertex0 == other.vertex0
            && self.vertex1 == other.vertex1
            && self.vertex2 == other.vertex2
    }
}

fn sort_three(v0: i32, v1: i32, v2: i32) -> (i32, i32, i32) {
    let (a, b) = if v0 < v1 { (v0, v1) } else { (v1, v0) };
    if v2 < a {
        (v2, a, b)
    } else if v2 < b {
        (a, v2, b)
    } else {
        (a, b, v2)
    }
}

struct OuterFaceInfo {
    vertex0: i32,
    vertex1: i32,
    vertex2: i32,
    tetrahedron: i32,
    index: i32,
    normal: Vec3,
    negative: f32,
    normal_probe_indices: [i32; 3],
}

#[derive(Debug, Clone)]
pub struct CircumSphere {
    pub center: Vec3,
    pub radius_squared: f32,
}

impl CircumSphere {
    pub fn new() -> Self {
        Self {
            center: Vec3::ZERO,
            radius_squared: 0.0,
        }
    }

    pub fn init(&mut self, p0: &Vec3, p1: &Vec3, p2: &Vec3, p3: &Vec3) {
        let m = Mat3::new(
            p1.x - p0.x,
            p1.y - p0.y,
            p1.z - p0.z,
            p2.x - p0.x,
            p2.y - p0.y,
            p2.z - p0.z,
            p3.x - p0.x,
            p3.y - p0.y,
            p3.z - p0.z,
        );
        let mut m_inv = m.get_inverted();
        m_inv.transpose();

        let n = Vec3::new(
            ((p1.x + p0.x) * (p1.x - p0.x)
                + (p1.y + p0.y) * (p1.y - p0.y)
                + (p1.z + p0.z) * (p1.z - p0.z))
                * 0.5,
            ((p2.x + p0.x) * (p2.x - p0.x)
                + (p2.y + p0.y) * (p2.y - p0.y)
                + (p2.z + p0.z) * (p2.z - p0.z))
                * 0.5,
            ((p3.x + p0.x) * (p3.x - p0.x)
                + (p3.y + p0.y) * (p3.y - p0.y)
                + (p3.z + p0.z) * (p3.z - p0.z))
                * 0.5,
        );

        self.center = m_inv.transform_vec3(&n);
        self.radius_squared = p0.distance_squared(&self.center);
    }
}

impl Default for CircumSphere {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Tetrahedron {
    pub invalid: bool,
    pub vertex0: i32,
    pub vertex1: i32,
    pub vertex2: i32,
    pub vertex3: i32,
    pub neighbours: [i32; 4],
    pub matrix: Mat3,
    pub offset: Vec3,
    pub sphere: CircumSphere,
}

impl Tetrahedron {
    pub fn new_inner(probes: &[Vertex], v0: i32, v1: i32, v2: i32, v3: i32) -> Self {
        let mut sphere = CircumSphere::new();
        if v3 >= 0 {
            sphere.init(
                &probes[v0 as usize].position,
                &probes[v1 as usize].position,
                &probes[v2 as usize].position,
                &probes[v3 as usize].position,
            );
        }
        Self {
            invalid: false,
            vertex0: v0,
            vertex1: v1,
            vertex2: v2,
            vertex3: v3,
            neighbours: [-1, -1, -1, -1],
            matrix: Mat3::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            offset: Vec3::ZERO,
            sphere,
        }
    }

    pub fn new_outer(v0: i32, v1: i32, v2: i32) -> Self {
        Self {
            invalid: false,
            vertex0: v0,
            vertex1: v1,
            vertex2: v2,
            vertex3: -1,
            neighbours: [-1, -1, -1, -1],
            matrix: Mat3::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            offset: Vec3::ZERO,
            sphere: CircumSphere::new(),
        }
    }

    pub fn is_in_circum_sphere(&self, point: &Vec3) -> bool {
        point.distance_squared(&self.sphere.center) < self.sphere.radius_squared - EPSILON
    }

    pub fn contain(&self, vertex_index: i32) -> bool {
        self.vertex0 == vertex_index
            || self.vertex1 == vertex_index
            || self.vertex2 == vertex_index
            || self.vertex3 == vertex_index
    }

    pub fn is_inner_tetrahedron(&self) -> bool {
        self.vertex3 >= 0
    }
    pub fn is_outer_cell(&self) -> bool {
        self.vertex3 < 0
    }
}

pub struct Delaunay {
    probes: Vec<Vertex>,
    tetrahedrons: Vec<Tetrahedron>,
    triangles: Vec<Triangle>,
    edges: Vec<Edge>,
}

impl Delaunay {
    pub fn new(probes: Vec<Vertex>) -> Self {
        Self {
            probes,
            tetrahedrons: Vec::new(),
            triangles: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn build(&mut self) -> Vec<Tetrahedron> {
        self.reset();
        self.tetrahedralize();
        self.compute_adjacency();
        self.compute_matrices();
        self.tetrahedrons.clone()
    }

    fn reset(&mut self) {
        self.tetrahedrons.clear();
        self.triangles.clear();
        self.edges.clear();
    }

    fn tetrahedralize(&mut self) {
        let probe_count = self.probes.len();
        let center = self.init_tetrahedron();
        for i in 0..probe_count {
            self.add_probe(i);
        }

        let vertex_index = probe_count as i32;
        self.tetrahedrons.retain(|t| {
            !(t.contain(vertex_index)
                || t.contain(vertex_index + 1)
                || t.contain(vertex_index + 2)
                || t.contain(vertex_index + 3))
        });
        self.probes.truncate(probe_count);
        self.reorder(&center);
    }

    fn init_tetrahedron(&mut self) -> Vec3 {
        let mut min_pos = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max_pos = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
        for probe in &self.probes {
            let p = probe.position;
            min_pos.x = min_pos.x.min(p.x);
            max_pos.x = max_pos.x.max(p.x);
            min_pos.y = min_pos.y.min(p.y);
            max_pos.y = max_pos.y.max(p.y);
            min_pos.z = min_pos.z.min(p.z);
            max_pos.z = max_pos.z.max(p.z);
        }
        let center = (min_pos + max_pos) * 0.5;
        let extent = max_pos - min_pos;
        let offset = extent.x.max(extent.y).max(extent.z) * 10.0;
        let p0 = Vec3::new(center.x, center.y + offset, center.z);
        let p1 = Vec3::new(center.x - offset, center.y - offset, center.z - offset);
        let p2 = Vec3::new(center.x - offset, center.y - offset, center.z + offset);
        let p3 = Vec3::new(center.x + offset, center.y - offset, center.z);
        let index = self.probes.len() as i32;
        self.probes.push(Vertex::new(p0));
        self.probes.push(Vertex::new(p1));
        self.probes.push(Vertex::new(p2));
        self.probes.push(Vertex::new(p3));
        self.tetrahedrons.push(Tetrahedron::new_inner(
            &self.probes,
            index,
            index + 1,
            index + 2,
            index + 3,
        ));
        center
    }

    #[allow(clippy::too_many_arguments)]
    fn add_triangle(&mut self, index: usize, tet: i32, i: i32, v0: i32, v1: i32, v2: i32, v3: i32) {
        if index < self.triangles.len() {
            self.triangles[index].set(tet, i, v0, v1, v2, v3);
        } else {
            self.triangles.push(Triangle::new(tet, i, v0, v1, v2, v3));
        }
    }

    fn add_probe(&mut self, vertex_index: usize) {
        let position = self.probes[vertex_index].position;
        let tet_data: Vec<(usize, i32, i32, i32, i32)> = self
            .tetrahedrons
            .iter()
            .enumerate()
            .filter(|(_, t)| t.is_in_circum_sphere(&position))
            .map(|(i, t)| (i, t.vertex0, t.vertex1, t.vertex2, t.vertex3))
            .collect();
        let mut triangle_index = 0;
        for (i, v0, v1, v2, v3) in &tet_data {
            self.tetrahedrons[*i].invalid = true;
            self.add_triangle(triangle_index, *i as i32, 0, *v1, *v3, *v2, *v0);
            self.add_triangle(triangle_index + 1, *i as i32, 1, *v0, *v2, *v3, *v1);
            self.add_triangle(triangle_index + 2, *i as i32, 2, *v0, *v3, *v1, *v2);
            self.add_triangle(triangle_index + 3, *i as i32, 3, *v0, *v1, *v2, *v3);
            triangle_index += 4;
        }
        for i in 0..triangle_index {
            if self.triangles[i].invalid {
                continue;
            }
            for k in (i + 1)..triangle_index {
                if self.triangles[i].is_same(&self.triangles[k]) {
                    self.triangles[i].invalid = true;
                    self.triangles[k].invalid = true;
                    break;
                }
            }
        }
        self.tetrahedrons.retain(|t| !t.invalid);
        for i in 0..triangle_index {
            if !self.triangles[i].invalid {
                let tri = &self.triangles[i];
                self.tetrahedrons.push(Tetrahedron::new_inner(
                    &self.probes,
                    tri.vertex0,
                    tri.vertex1,
                    tri.vertex2,
                    vertex_index as i32,
                ));
            }
        }
    }

    fn reorder(&mut self, center: &Vec3) {
        self.tetrahedrons.sort_by(|a, b| {
            let da = a.sphere.center.distance_squared(center);
            let db = b.sphere.center.distance_squared(center);
            db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    fn compute_adjacency(&mut self) {
        let tetrahedron_count = self.tetrahedrons.len();
        self.triangles.clear();
        for i in 0..tetrahedron_count {
            let tet = &self.tetrahedrons[i];
            self.triangles.push(Triangle::new(
                i as i32,
                0,
                tet.vertex1,
                tet.vertex3,
                tet.vertex2,
                tet.vertex0,
            ));
            self.triangles.push(Triangle::new(
                i as i32,
                1,
                tet.vertex0,
                tet.vertex2,
                tet.vertex3,
                tet.vertex1,
            ));
            self.triangles.push(Triangle::new(
                i as i32,
                2,
                tet.vertex0,
                tet.vertex3,
                tet.vertex1,
                tet.vertex2,
            ));
            self.triangles.push(Triangle::new(
                i as i32,
                3,
                tet.vertex0,
                tet.vertex1,
                tet.vertex2,
                tet.vertex3,
            ));
        }

        let triangle_count = self.triangles.len();
        let mut neighbour_updates: Vec<(usize, usize, i32)> = Vec::new();
        let mut outer_face_data: Vec<OuterFaceInfo> = Vec::new();

        for i in 0..triangle_count {
            if !self.triangles[i].is_outer_face {
                continue;
            }
            let mut found_match = false;
            for k in (i + 1)..triangle_count {
                if self.triangles[i].is_same(&self.triangles[k]) {
                    neighbour_updates.push((
                        self.triangles[i].tetrahedron as usize,
                        self.triangles[i].index as usize,
                        self.triangles[k].tetrahedron,
                    ));
                    neighbour_updates.push((
                        self.triangles[k].tetrahedron as usize,
                        self.triangles[k].index as usize,
                        self.triangles[i].tetrahedron,
                    ));
                    self.triangles[i].is_outer_face = false;
                    self.triangles[k].is_outer_face = false;
                    found_match = true;
                    break;
                }
            }
            if !found_match && self.triangles[i].is_outer_face {
                let vi = &self.triangles[i];
                let p0 = self.probes[vi.vertex0 as usize].position;
                let p1 = self.probes[vi.vertex1 as usize].position;
                let p2 = self.probes[vi.vertex2 as usize].position;
                let p3 = self.probes[vi.vertex3 as usize].position;
                let edge1 = p1 - p0;
                let edge2 = p2 - p0;
                let mut normal = Vec3::cross_vecs(&edge1, &edge2);
                let edge3 = p3 - p0;
                let negative = Vec3::dot_vecs(&normal, &edge3);
                if negative > 0.0 {
                    normal = -normal;
                }
                outer_face_data.push(OuterFaceInfo {
                    vertex0: vi.vertex0,
                    vertex1: vi.vertex1,
                    vertex2: vi.vertex2,
                    tetrahedron: vi.tetrahedron,
                    index: vi.index,
                    normal,
                    negative,
                    normal_probe_indices: [vi.vertex0, vi.vertex1, vi.vertex2],
                });
            }
        }

        for (ti, ii, neighbour) in &neighbour_updates {
            self.tetrahedrons[*ti].neighbours[*ii] = *neighbour;
        }

        for info in &outer_face_data {
            for idx in &info.normal_probe_indices {
                #[allow(clippy::assign_op_pattern)]
                {
                    self.probes[*idx as usize].normal =
                        self.probes[*idx as usize].normal + info.normal;
                }
            }
            let v0 = info.vertex0;
            let v1 = if info.negative > 0.0 {
                info.vertex2
            } else {
                info.vertex1
            };
            let v2 = if info.negative > 0.0 {
                info.vertex1
            } else {
                info.vertex2
            };
            let mut outer_tet = Tetrahedron::new_outer(v0, v1, v2);
            outer_tet.neighbours[3] = info.tetrahedron;
            let new_idx = self.tetrahedrons.len() as i32;
            self.tetrahedrons[info.tetrahedron as usize].neighbours[info.index as usize] = new_idx;
            self.tetrahedrons.push(outer_tet);
        }

        self.edges.clear();
        for i in tetrahedron_count..self.tetrahedrons.len() {
            let tet = &self.tetrahedrons[i];
            self.edges
                .push(Edge::new(i as i32, 0, tet.vertex1, tet.vertex2));
            self.edges
                .push(Edge::new(i as i32, 1, tet.vertex2, tet.vertex0));
            self.edges
                .push(Edge::new(i as i32, 2, tet.vertex0, tet.vertex1));
        }
        let edge_count = self.edges.len();
        for i in 0..edge_count {
            for k in (i + 1)..edge_count {
                if self.edges[i].is_same(&self.edges[k]) {
                    let ei = self.edges[i].tetrahedron as usize;
                    let ek = self.edges[k].tetrahedron as usize;
                    let ii = self.edges[i].index as usize;
                    let ik = self.edges[k].index as usize;
                    self.tetrahedrons[ei].neighbours[ii] = self.edges[k].tetrahedron;
                    self.tetrahedrons[ek].neighbours[ik] = self.edges[i].tetrahedron;
                }
            }
        }
        for i in 0..self.probes.len() {
            self.probes[i].normal.normalize();
        }
    }

    fn compute_matrices(&mut self) {
        for i in 0..self.tetrahedrons.len() {
            if self.tetrahedrons[i].vertex3 >= 0 {
                self.compute_tetrahedron_matrix(i);
            } else {
                self.compute_outer_cell_matrix(i);
            }
        }
    }

    fn compute_tetrahedron_matrix(&mut self, idx: usize) {
        let tet = &self.tetrahedrons[idx];
        let p0 = self.probes[tet.vertex0 as usize].position;
        let p1 = self.probes[tet.vertex1 as usize].position;
        let p2 = self.probes[tet.vertex2 as usize].position;
        let p3 = self.probes[tet.vertex3 as usize].position;
        let mut m = Mat3::new(
            p0.x - p3.x,
            p1.x - p3.x,
            p2.x - p3.x,
            p0.y - p3.y,
            p1.y - p3.y,
            p2.y - p3.y,
            p0.z - p3.z,
            p1.z - p3.z,
            p2.z - p3.z,
        );
        m.invert();
        m.transpose();
        self.tetrahedrons[idx].matrix = m;
    }

    fn compute_outer_cell_matrix(&mut self, idx: usize) {
        let tet = &self.tetrahedrons[idx];
        let v0 = &self.probes[tet.vertex0 as usize];
        let v1 = &self.probes[tet.vertex1 as usize];
        let v2 = &self.probes[tet.vertex2 as usize];
        let a = v0.position - v2.position;
        let ap = v0.normal - v2.normal;
        let b = v1.position - v2.position;
        let bp = v1.normal - v2.normal;
        let p2 = v2.position;
        let cp = -v2.normal;
        let m: [f32; 12] = compute_outer_cell_matrix_values(&a, &ap, &b, &bp, &p2, &cp);
        let c = ap.x * bp.y * cp.z - ap.y * bp.x * cp.z + ap.z * bp.x * cp.y - ap.z * bp.y * cp.x
            + ap.y * bp.z * cp.x
            - ap.x * bp.z * cp.y;
        let mut m_vals = m;
        if c.abs() > EPSILON {
            for k in 0..12 {
                m_vals[k] /= c;
            }
        } else {
            self.tetrahedrons[idx].vertex3 = -2;
        }
        self.tetrahedrons[idx].matrix = Mat3::new(
            m_vals[0], m_vals[1], m_vals[2], m_vals[3], m_vals[4], m_vals[5], m_vals[6], m_vals[7],
            m_vals[8],
        );
        self.tetrahedrons[idx].offset = Vec3::new(m_vals[9], m_vals[10], m_vals[11]);
    }
}

fn compute_outer_cell_matrix_values(
    a: &Vec3,
    ap: &Vec3,
    b: &Vec3,
    bp: &Vec3,
    p2: &Vec3,
    cp: &Vec3,
) -> [f32; 12] {
    let m0 = ap.y * bp.z - ap.z * bp.y;
    let m3 = -ap.x * bp.z + ap.z * bp.x;
    let m6 = ap.x * bp.y - ap.y * bp.x;
    let m9 = a.x * bp.y * cp.z - a.y * bp.x * cp.z + ap.x * b.y * cp.z - ap.y * b.x * cp.z
        + a.z * bp.x * cp.y
        - a.z * bp.y * cp.x
        + ap.z * b.x * cp.y
        - ap.z * b.y * cp.x
        - a.x * bp.z * cp.y
        + a.y * bp.z * cp.x
        - ap.x * b.z * cp.y
        + ap.y * b.z * cp.x;
    let m9 = m9 - (p2.x * m0 + p2.y * m3 + p2.z * m6);
    let m1 = ap.y * b.z + a.y * bp.z - ap.z * b.y - a.z * bp.y;
    let m4 = -a.x * bp.z - ap.x * b.z + a.z * bp.x + ap.z * b.x;
    let m7 = a.x * bp.y - a.y * bp.x + ap.x * b.y - ap.y * b.x;
    let m10 = a.x * b.y * cp.z - a.y * b.x * cp.z - a.x * b.z * cp.y
        + a.y * b.z * cp.x
        + a.z * b.x * cp.y
        - a.z * b.y * cp.x;
    let m10 = m10 - (p2.x * m1 + p2.y * m4 + p2.z * m7);
    let m2 = -a.z * b.y + a.y * b.z;
    let m5 = -a.x * b.z + a.z * b.x;
    let m8 = a.x * b.y - a.y * b.x;
    let m11 = -(p2.x * m2 + p2.y * m5 + p2.z * m8);
    [m0, m1, m2, m3, m4, m5, m6, m7, m8, m9, m10, m11]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_new() {
        let v = Vertex::new(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(v.position.x, 1.0);
        assert_eq!(v.coefficients.len(), SH_BASIS_COUNT);
    }

    #[test]
    fn test_tetrahedron_inner() {
        let probes = vec![
            Vertex::new(Vec3::ZERO),
            Vertex::new(Vec3::UNIT_X),
            Vertex::new(Vec3::UNIT_Y),
            Vertex::new(Vec3::UNIT_Z),
        ];
        let tet = Tetrahedron::new_inner(&probes, 0, 1, 2, 3);
        assert!(tet.is_inner_tetrahedron());
        assert!(!tet.is_outer_cell());
        assert!(tet.contain(0));
        assert!(!tet.contain(5));
    }

    #[test]
    fn test_tetrahedron_outer() {
        let tet = Tetrahedron::new_outer(0, 1, 2);
        assert!(tet.is_outer_cell());
        assert!(!tet.is_inner_tetrahedron());
        assert_eq!(tet.vertex3, -1);
    }

    #[test]
    fn test_circum_sphere() {
        let p0 = Vec3::ZERO;
        let p1 = Vec3::UNIT_X;
        let p2 = Vec3::UNIT_Y;
        let p3 = Vec3::UNIT_Z;
        let mut sphere = CircumSphere::new();
        sphere.init(&p0, &p1, &p2, &p3);
        assert!(sphere.radius_squared > 0.0);
    }

    #[test]
    fn test_delaunay_build() {
        let probes = vec![
            Vertex::new(Vec3::ZERO),
            Vertex::new(Vec3::UNIT_X),
            Vertex::new(Vec3::UNIT_Y),
            Vertex::new(Vec3::UNIT_Z),
            Vertex::new(Vec3::ONE),
        ];
        let mut delaunay = Delaunay::new(probes);
        let tets = delaunay.build();
        assert!(!tets.is_empty());
        assert!(tets.iter().any(|t| t.is_inner_tetrahedron()));
    }

    #[test]
    fn test_sort_three() {
        assert_eq!(sort_three(3, 1, 2), (1, 2, 3));
        assert_eq!(sort_three(0, 2, 1), (0, 1, 2));
    }
}
