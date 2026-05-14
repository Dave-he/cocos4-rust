/****************************************************************************
Rust port of Cocos Creator PassInsertPointManager
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::pass::PassInsertPoint;
use std::collections::HashMap;

pub struct PassInsertPointManager {
    string_pool: HashMap<String, u32>,
    insert_points: Vec<PassInsertPoint>,
    names: Vec<String>,
}

impl PassInsertPointManager {
    pub fn new() -> Self {
        PassInsertPointManager {
            string_pool: HashMap::new(),
            insert_points: Vec::new(),
            names: Vec::new(),
        }
    }

    pub fn record(&mut self, name: &str, point: PassInsertPoint) -> PassInsertPoint {
        if let Some(&idx) = self.string_pool.get(name) {
            self.insert_points[idx as usize] = point;
            point
        } else {
            let idx = self.insert_points.len() as u32;
            self.string_pool.insert(name.to_string(), idx);
            self.insert_points.push(point);
            self.names.push(name.to_string());
            point
        }
    }

    pub fn get(&self, name: &str) -> Option<PassInsertPoint> {
        self.string_pool
            .get(name)
            .and_then(|&idx| self.insert_points.get(idx as usize))
            .copied()
    }

    pub fn get_or_default(&self, name: &str, default_point: PassInsertPoint) -> PassInsertPoint {
        self.get(name).unwrap_or(default_point)
    }

    pub fn get_count(&self) -> usize {
        self.insert_points.len()
    }

    pub fn get_name(&self, index: usize) -> Option<&str> {
        self.names.get(index).map(|s| s.as_str())
    }

    pub fn get_insert_point(&self, index: usize) -> Option<PassInsertPoint> {
        self.insert_points.get(index).copied()
    }

    pub fn clear(&mut self) {
        self.string_pool.clear();
        self.insert_points.clear();
        self.names.clear();
    }
}

impl Default for PassInsertPointManager {
    fn default() -> Self {
        Self::new()
    }
}

pub const INSERT_POINT_FORWARD: PassInsertPoint = 0;
pub const INSERT_POINT_SHADOW: PassInsertPoint = 1;
pub const INSERT_POINT_POST_PROCESS: PassInsertPoint = 2;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_point_manager_record() {
        let mut mgr = PassInsertPointManager::new();
        mgr.record("Forward", INSERT_POINT_FORWARD);
        mgr.record("Shadow", INSERT_POINT_SHADOW);
        assert_eq!(mgr.get_count(), 2);
    }

    #[test]
    fn test_insert_point_manager_get() {
        let mut mgr = PassInsertPointManager::new();
        mgr.record("Forward", INSERT_POINT_FORWARD);
        assert_eq!(mgr.get("Forward"), Some(INSERT_POINT_FORWARD));
        assert_eq!(mgr.get("Unknown"), None);
    }

    #[test]
    fn test_insert_point_manager_get_or_default() {
        let mut mgr = PassInsertPointManager::new();
        mgr.record("Forward", INSERT_POINT_FORWARD);
        assert_eq!(mgr.get_or_default("Forward", 99), INSERT_POINT_FORWARD);
        assert_eq!(mgr.get_or_default("Unknown", 99), 99);
    }

    #[test]
    fn test_insert_point_manager_update() {
        let mut mgr = PassInsertPointManager::new();
        mgr.record("Forward", 0);
        mgr.record("Forward", 10);
        assert_eq!(mgr.get("Forward"), Some(10));
        assert_eq!(mgr.get_count(), 1);
    }

    #[test]
    fn test_insert_point_manager_clear() {
        let mut mgr = PassInsertPointManager::new();
        mgr.record("Forward", 0);
        mgr.record("Shadow", 1);
        mgr.clear();
        assert_eq!(mgr.get_count(), 0);
        assert_eq!(mgr.get("Forward"), None);
    }

    #[test]
    fn test_insert_point_manager_name_lookup() {
        let mut mgr = PassInsertPointManager::new();
        mgr.record("Forward", 0);
        mgr.record("Shadow", 1);
        assert_eq!(mgr.get_name(0), Some("Forward"));
        assert_eq!(mgr.get_name(1), Some("Shadow"));
        assert_eq!(mgr.get_insert_point(0), Some(0));
        assert_eq!(mgr.get_insert_point(1), Some(1));
    }
}
