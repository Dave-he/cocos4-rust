/****************************************************************************
Rust port of Cocos Creator Spherical Harmonics (SH)
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/
// SPDX-License-Identifier: MIT

use crate::math::Vec3;

pub const SH_BASIS_COUNT: usize = 9;
const PI: f32 = std::f32::consts::PI;

pub fn evaluate_basis(index: usize, sample: &Vec3) -> f32 {
    let x = sample.x;
    let y = sample.y;
    let z = sample.z;
    match index {
        0 => 0.282095,
        1 => -0.488603 * y,
        2 => 0.488603 * z,
        3 => -0.488603 * x,
        4 => 1.092548 * x * y,
        5 => -1.092548 * y * z,
        6 => 0.315392 * (3.0 * z * z - 1.0),
        7 => -1.092548 * x * z,
        8 => 0.546274 * (x * x - y * y),
        _ => 0.0,
    }
}

pub fn evaluate(sample: &Vec3, coefficients: &[Vec3]) -> Vec3 {
    let mut result = Vec3::new(0.0, 0.0, 0.0);
    for i in 0..SH_BASIS_COUNT.min(coefficients.len()) {
        let basis = evaluate_basis(i, sample);
        result.x += coefficients[i].x * basis;
        result.y += coefficients[i].y * basis;
        result.z += coefficients[i].z * basis;
    }
    result
}

pub fn project(samples: &[Vec3], values: &[Vec3]) -> Vec<Vec3> {
    let weight = 4.0 * PI / samples.len() as f32;
    let mut coefficients = vec![Vec3::new(0.0, 0.0, 0.0); SH_BASIS_COUNT];
    for (sample, value) in samples.iter().zip(values.iter()) {
        #[allow(clippy::needless_range_loop)]
        for i in 0..SH_BASIS_COUNT {
            let basis = evaluate_basis(i, sample);
            coefficients[i].x += value.x * basis * weight;
            coefficients[i].y += value.y * basis * weight;
            coefficients[i].z += value.z * basis * weight;
        }
    }
    coefficients
}

pub fn convolve_cosine(radiance_coefficients: &[Vec3]) -> Vec<Vec3> {
    let cosine_kernel = [
        0.282095, 0.488603, 0.488603, 0.488603, 1.092548, 1.092548, 0.315392, 1.092548, 0.546274,
    ];
    let mut irradiance = vec![Vec3::new(0.0, 0.0, 0.0); SH_BASIS_COUNT];
    for i in 0..SH_BASIS_COUNT.min(radiance_coefficients.len()) {
        irradiance[i] = radiance_coefficients[i] * cosine_kernel[i];
    }
    irradiance
}

pub fn reduce_ringing(coefficients: &mut [Vec3], lambda: f32) {
    if coefficients.len() < SH_BASIS_COUNT {
        return;
    }
    let l0_factor = 1.0;
    let l1_factor = 1.0;
    let l2_factor = 1.0 - lambda;
    let scales = [
        l0_factor, l1_factor, l1_factor, l1_factor, l2_factor, l2_factor, l2_factor, l2_factor,
        l2_factor,
    ];
    #[allow(clippy::assign_op_pattern)]
    for i in 0..SH_BASIS_COUNT {
        coefficients[i] = coefficients[i] * scales[i];
    }
}

pub fn shader_evaluate(normal: &Vec3, coefficients: &[Vec3]) -> Vec3 {
    evaluate(normal, coefficients)
}

pub fn update_ubo_data(data: &mut [f32], offset: usize, coefficients: &[Vec3]) {
    #[allow(clippy::needless_range_loop)]
    for i in 0..SH_BASIS_COUNT.min(coefficients.len()) {
        let idx = offset + i * 3;
        if idx + 2 < data.len() {
            data[idx] = coefficients[i].x;
            data[idx + 1] = coefficients[i].y;
            data[idx + 2] = coefficients[i].z;
        }
    }
}

pub struct LightProbeSampler;

impl LightProbeSampler {
    pub fn uniform_sample_sphere(u1: f32, u2: f32) -> Vec3 {
        let z = 1.0 - 2.0 * u1;
        let r = (1.0 - z * z).max(0.0).sqrt();
        let phi = 2.0 * PI * u2;
        Vec3::new(r * phi.cos(), r * phi.sin(), z)
    }

    pub fn uniform_sample_sphere_all(sample_count: u32) -> Vec<Vec3> {
        let mut samples = Vec::with_capacity(sample_count as usize);
        #[allow(clippy::excessive_precision)]
        let golden_ratio = 0.618033988749895_f32;
        for i in 0..sample_count {
            let u1 = (i as f32 + 0.5) / sample_count as f32;
            let u2 = ((i as f32 + 0.5) * golden_ratio) % 1.0;
            samples.push(Self::uniform_sample_sphere(u1, u2));
        }
        samples
    }

    pub fn uniform_sphere_pdf() -> f32 {
        1.0 / (4.0 * PI)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sh_basis_count() {
        assert_eq!(SH_BASIS_COUNT, 9);
    }

    #[test]
    fn test_evaluate_basis_l0() {
        let v = Vec3::new(1.0, 0.0, 0.0);
        let b0 = evaluate_basis(0, &v);
        assert!((b0 - 0.282095).abs() < 0.001);
    }

    #[test]
    fn test_evaluate_basis_l1() {
        let v = Vec3::new(0.0, 1.0, 0.0);
        let b1 = evaluate_basis(1, &v);
        assert!((b1 + 0.488603).abs() < 0.001);
    }

    #[test]
    fn test_uniform_sample_sphere() {
        let s = LightProbeSampler::uniform_sample_sphere(0.5, 0.25);
        assert!(s.x.abs() < 1.01 && s.y.abs() < 1.01 && s.z.abs() < 1.01);
    }

    #[test]
    fn test_uniform_sphere_pdf() {
        let pdf = LightProbeSampler::uniform_sphere_pdf();
        assert!((pdf - 1.0 / (4.0 * PI)).abs() < 0.001);
    }

    #[test]
    fn test_project_and_evaluate() {
        let samples = LightProbeSampler::uniform_sample_sphere_all(64);
        let values: Vec<Vec3> = samples.iter().map(|_s| Vec3::new(1.0, 1.0, 1.0)).collect();
        let coeffs = project(&samples, &values);
        let result = evaluate(&Vec3::new(0.0, 0.0, 1.0), &coeffs);
        assert!(result.x > 0.0);
    }

    #[test]
    fn test_convolve_cosine() {
        let radiance = vec![Vec3::new(1.0, 1.0, 1.0); SH_BASIS_COUNT];
        let irradiance = convolve_cosine(&radiance);
        assert!(irradiance[0].x > 0.0);
    }

    #[test]
    fn test_reduce_ringing() {
        let mut coeffs = vec![Vec3::new(1.0, 1.0, 1.0); SH_BASIS_COUNT];
        reduce_ringing(&mut coeffs, 0.5);
        assert!((coeffs[4].x - 0.5).abs() < 0.001);
    }
}
