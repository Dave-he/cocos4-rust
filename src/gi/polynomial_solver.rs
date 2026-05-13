/****************************************************************************
Rust port of Cocos Creator Polynomial Solver
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/
// SPDX-License-Identifier: MIT

pub struct PolynomialSolver;

impl PolynomialSolver {
    pub fn get_quadratic_unique_root(b: f32, c: f32, d: f32) -> f32 {
        if b != 0.0 {
            -c / (2.0 * b)
        } else if c != 0.0 {
            -d / c
        } else {
            0.0
        }
    }

    pub fn get_cubic_unique_root(b: f32, c: f32, d: f32) -> f32 {
        let offset = -b / 3.0;
        let p = c / 3.0 - (b * b) / 9.0;
        let q = d / 2.0 + (b * b * b) / 27.0 - (b * c) / 6.0;
        let delta = p * p * p + q * q;

        let mut roots: Vec<f32> = Vec::new();

        if delta > 0.0 {
            let sqrt_delta = delta.sqrt();
            roots.push((-q + sqrt_delta).cbrt() + (-q - sqrt_delta).cbrt());
        } else if delta < 0.0 {
            let angle = (-q * (-p).sqrt() / (p * p)).acos() / 3.0;
            roots.push(2.0 * (-p).sqrt() * angle.cos());
            roots.push(2.0 * (-p).sqrt() * (angle + 2.0 * std::f32::consts::PI / 3.0).cos());
            roots.push(2.0 * (-p).sqrt() * (angle + 4.0 * std::f32::consts::PI / 3.0).cos());
        } else if q == 0.0 {
            roots.push(0.0);
        } else {
            let root = q.cbrt();
            roots.push(root);
            roots.push(-2.0 * root);
        }

        for root in &roots {
            if *root + offset >= 0.0 {
                return *root + offset;
            }
        }

        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quadratic_simple() {
        let result = PolynomialSolver::get_quadratic_unique_root(1.0, -2.0, 1.0);
        assert!((result - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_quadratic_linear() {
        let result = PolynomialSolver::get_quadratic_unique_root(0.0, 2.0, -4.0);
        assert!((result - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_cubic_single_root() {
        let result = PolynomialSolver::get_cubic_unique_root(0.0, 0.0, -1.0);
        assert!(result >= 0.0);
    }

    #[test]
    fn test_cubic_three_roots() {
        let result = PolynomialSolver::get_cubic_unique_root(0.0, -3.0, 2.0);
        assert!(result >= 0.0);
    }
}
