/****************************************************************************
Rust port of Cocos Creator GI module
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/
// SPDX-License-Identifier: MIT

pub mod auto_placement;
pub mod delaunay;
pub mod light_probe;
pub mod polynomial_solver;
pub mod sh;

pub use auto_placement::{AutoPlacement, PlaceMethod, PlacementInfo};
pub use delaunay::{CircumSphere, Delaunay, Tetrahedron, Vertex};
pub use light_probe::{LightProbeInfo, LightProbes, LightProbesData};
pub use polynomial_solver::PolynomialSolver;
pub use sh::{
    convolve_cosine, evaluate, evaluate_basis, project, reduce_ringing, shader_evaluate,
    update_ubo_data, LightProbeSampler, SH_BASIS_COUNT,
};
