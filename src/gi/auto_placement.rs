/****************************************************************************
Rust port of Cocos Creator Auto Placement
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/
// SPDX-License-Identifier: MIT

use crate::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceMethod {
    Uniform = 0,
    Adaptive = 1,
}

#[derive(Debug, Clone)]
pub struct PlacementInfo {
    pub method: PlaceMethod,
    pub n_probes_x: u32,
    pub n_probes_y: u32,
    pub n_probes_z: u32,
    pub min_pos: Vec3,
    pub max_pos: Vec3,
}

pub struct AutoPlacement;

impl AutoPlacement {
    pub fn generate(info: &PlacementInfo) -> Vec<Vec3> {
        match info.method {
            PlaceMethod::Uniform => Self::do_generate_uniform(info),
            PlaceMethod::Adaptive => Self::do_generate_adaptive(info),
        }
    }

    fn do_generate_uniform(info: &PlacementInfo) -> Vec<Vec3> {
        if info.n_probes_x < 2 || info.n_probes_y < 2 || info.n_probes_z < 2 {
            return Vec::new();
        }

        let grid_size = Vec3::new(
            (info.max_pos.x - info.min_pos.x) / (info.n_probes_x - 1) as f32,
            (info.max_pos.y - info.min_pos.y) / (info.n_probes_y - 1) as f32,
            (info.max_pos.z - info.min_pos.z) / (info.n_probes_z - 1) as f32,
        );

        let mut probes = Vec::new();
        for x in 0..info.n_probes_x {
            for y in 0..info.n_probes_y {
                for z in 0..info.n_probes_z {
                    probes.push(Vec3::new(
                        x as f32 * grid_size.x + info.min_pos.x,
                        y as f32 * grid_size.y + info.min_pos.y,
                        z as f32 * grid_size.z + info.min_pos.z,
                    ));
                }
            }
        }

        probes
    }

    fn do_generate_adaptive(info: &PlacementInfo) -> Vec<Vec3> {
        Self::do_generate_uniform(info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_placement() {
        let info = PlacementInfo {
            method: PlaceMethod::Uniform,
            n_probes_x: 3,
            n_probes_y: 3,
            n_probes_z: 3,
            min_pos: Vec3::new(-1.0, -1.0, -1.0),
            max_pos: Vec3::new(1.0, 1.0, 1.0),
        };
        let probes = AutoPlacement::generate(&info);
        assert_eq!(probes.len(), 27);
    }

    #[test]
    fn test_uniform_placement_too_small() {
        let info = PlacementInfo {
            method: PlaceMethod::Uniform,
            n_probes_x: 1,
            n_probes_y: 2,
            n_probes_z: 2,
            min_pos: Vec3::new(0.0, 0.0, 0.0),
            max_pos: Vec3::new(1.0, 1.0, 1.0),
        };
        let probes = AutoPlacement::generate(&info);
        assert!(probes.is_empty());
    }

    #[test]
    fn test_adaptive_placement() {
        let info = PlacementInfo {
            method: PlaceMethod::Adaptive,
            n_probes_x: 2,
            n_probes_y: 2,
            n_probes_z: 2,
            min_pos: Vec3::new(0.0, 0.0, 0.0),
            max_pos: Vec3::new(2.0, 2.0, 2.0),
        };
        let probes = AutoPlacement::generate(&info);
        assert_eq!(probes.len(), 8);
    }

    #[test]
    fn test_place_method_equality() {
        assert_eq!(PlaceMethod::Uniform, PlaceMethod::Uniform);
        assert_ne!(PlaceMethod::Uniform, PlaceMethod::Adaptive);
    }
}
