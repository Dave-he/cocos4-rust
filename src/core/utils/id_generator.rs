/****************************************************************************
Rust port of Cocos Creator IDGenerator
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/
// SPDX-License-Identifier: MIT

use crate::base::random::random_range_i;
use std::sync::atomic::{AtomicU32, Ordering};

pub const NON_UUID_MARK: &str = ".";

pub struct IDGenerator {
    id: AtomicU32,
    prefix: String,
}

impl IDGenerator {
    pub fn new(category: &str) -> Self {
        let init_id = random_range_i(0, 998) as u32;
        IDGenerator {
            id: AtomicU32::new(init_id),
            prefix: category.to_string() + NON_UUID_MARK,
        }
    }

    pub fn get_new_id(&self) -> String {
        let next = self.id.fetch_add(1, Ordering::Relaxed) + 1;
        format!("{}{}", self.prefix, next)
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_ID_GENERATOR: IDGenerator = IDGenerator::new("global");
}

pub fn get_new_global_id() -> String {
    GLOBAL_ID_GENERATOR.get_new_id()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_generator_basic() {
        let gen = IDGenerator::new("test");
        let id1 = gen.get_new_id();
        let id2 = gen.get_new_id();
        assert!(id1.starts_with("test."));
        assert!(id2.starts_with("test."));
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_id_generator_sequential() {
        let gen = IDGenerator::new("node");
        let ids: Vec<String> = (0..10).map(|_| gen.get_new_id()).collect();
        for i in 1..ids.len() {
            let prev_num: u32 = ids[i - 1].split('.').last().unwrap().parse().unwrap();
            let curr_num: u32 = ids[i].split('.').last().unwrap().parse().unwrap();
            assert_eq!(curr_num, prev_num + 1);
        }
    }

    #[test]
    fn test_non_uuid_mark() {
        assert_eq!(NON_UUID_MARK, ".");
    }

    #[test]
    fn test_global_id_generator() {
        let id = get_new_global_id();
        assert!(id.starts_with("global."));
    }
}
