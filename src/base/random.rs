/****************************************************************************
Rust port of Cocos Creator RandomHelper
Original C++ version Copyright (c) 2017-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/
// SPDX-License-Identifier: MIT

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::cell::RefCell;

thread_local! {
    static RNG: RefCell<StdRng> = RefCell::new(StdRng::from_entropy());
}

pub fn random_range_i(min: i32, max: i32) -> i32 {
    RNG.with(|rng| rng.borrow_mut().gen_range(min..=max))
}

pub fn random_range_f(min: f32, max: f32) -> f32 {
    RNG.with(|rng| rng.borrow_mut().gen_range(min..max))
}

pub fn random_range_d(min: f64, max: f64) -> f64 {
    RNG.with(|rng| rng.borrow_mut().gen_range(min..max))
}

pub fn rand_minus1_1() -> f32 {
    random_range_f(-1.0, 1.0)
}

pub fn rand_0_1() -> f32 {
    random_range_f(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_range_i() {
        let val = random_range_i(0, 100);
        assert!((0..=100).contains(&val));
    }

    #[test]
    fn test_random_range_f() {
        let val = random_range_f(0.0, 1.0);
        assert!((0.0..1.0).contains(&val));
    }

    #[test]
    fn test_random_range_d() {
        let val = random_range_d(0.0, 1.0);
        assert!((0.0..1.0).contains(&val));
    }

    #[test]
    fn test_rand_minus1_1() {
        let val = rand_minus1_1();
        assert!((-1.0..1.0).contains(&val));
    }

    #[test]
    fn test_rand_0_1() {
        let val = rand_0_1();
        assert!((0.0..1.0).contains(&val));
    }

    #[test]
    fn test_random_range_i_bounds() {
        for _ in 0..100 {
            let val = random_range_i(-10, 10);
            assert!((-10..=10).contains(&val));
        }
    }

    #[test]
    fn test_random_range_f_distribution() {
        let mut count = 0;
        for _ in 0..1000 {
            let val = random_range_f(0.0, 1.0);
            if val > 0.5 {
                count += 1;
            }
        }
        assert!(count > 300 && count < 700);
    }
}
