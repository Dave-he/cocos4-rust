/****************************************************************************
Rust port of Cocos Creator Blackboard System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HandleIndexType {
    Uninitialized = 0,
    IndexType = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct StringHandle {
    pub index: u32,
    pub index_type: u32,
}

impl StringHandle {
    pub const INVALID: StringHandle = StringHandle { index: u32::MAX, index_type: 0 };

    pub fn new(index: u32) -> Self {
        StringHandle { index, index_type: 1 }
    }

    pub fn is_valid(&self) -> bool {
        self.index != u32::MAX
    }
}

#[derive(Debug)]
pub struct Blackboard<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone + Default,
{
    container: HashMap<K, V>,
    invalid_value: V,
}

impl<K, V> Blackboard<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone + Default,
{
    pub fn new(invalid_value: V) -> Self {
        Blackboard {
            container: HashMap::new(),
            invalid_value,
        }
    }

    pub fn put(&mut self, name: K, value: V) {
        self.container.insert(name, value);
    }

    pub fn get(&self, name: &K) -> V {
        self.container
            .get(name)
            .cloned()
            .unwrap_or_else(|| self.invalid_value.clone())
    }

    pub fn get_or_insert(&mut self, name: K) -> &mut V {
        let invalid = self.invalid_value.clone();
        self.container.entry(name).or_insert(invalid)
    }

    pub fn clear(&mut self) {
        self.container.clear();
    }

    pub fn has(&self, name: &K) -> bool {
        self.container.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.container.len()
    }

    pub fn is_empty(&self) -> bool {
        self.container.is_empty()
    }
}

pub type FrameGraphBlackboard = Blackboard<String, u32>;

impl FrameGraphBlackboard {
    pub fn default_board() -> Self {
        Blackboard::new(u32::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_handle_valid() {
        let h = StringHandle::new(0);
        assert!(h.is_valid());
        assert!(!StringHandle::INVALID.is_valid());
    }

    #[test]
    fn test_blackboard_put_get() {
        let mut board: Blackboard<String, u32> = Blackboard::new(u32::MAX);
        board.put("color".to_string(), 42);
        assert_eq!(board.get(&"color".to_string()), 42);
        assert_eq!(board.get(&"missing".to_string()), u32::MAX);
    }

    #[test]
    fn test_blackboard_has() {
        let mut board: Blackboard<String, u32> = Blackboard::new(u32::MAX);
        assert!(!board.has(&"key".to_string()));
        board.put("key".to_string(), 1);
        assert!(board.has(&"key".to_string()));
    }

    #[test]
    fn test_blackboard_clear() {
        let mut board: Blackboard<String, u32> = Blackboard::new(0);
        board.put("a".to_string(), 1);
        board.put("b".to_string(), 2);
        assert_eq!(board.len(), 2);
        board.clear();
        assert!(board.is_empty());
    }

    #[test]
    fn test_blackboard_get_or_insert() {
        let mut board: Blackboard<String, u32> = Blackboard::new(0);
        let val = board.get_or_insert("new_key".to_string());
        *val = 99;
        assert_eq!(board.get(&"new_key".to_string()), 99);
    }

    #[test]
    fn test_frame_graph_blackboard() {
        let mut board = FrameGraphBlackboard::default_board();
        board.put("depth".to_string(), 5);
        assert_eq!(board.get(&"depth".to_string()), 5);
        assert_eq!(board.get(&"missing".to_string()), u32::MAX);
    }
}
