use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Local storage key-value store
pub struct LocalStorage {
    data: Arc<RwLock<HashMap<String, String>>>,
}

impl LocalStorage {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get an item from storage
    pub fn get_item(&self, key: &str) -> Option<String> {
        let data = self.data.read().unwrap();
        data.get(key).cloned()
    }

    /// Set an item in storage
    pub fn set_item(&self, key: &str, value: &str) {
        let mut data = self.data.write().unwrap();
        data.insert(key.to_string(), value.to_string());
    }

    /// Remove an item from storage
    pub fn remove_item(&self, key: &str) {
        let mut data = self.data.write().unwrap();
        data.remove(key);
    }

    /// Clear all items from storage
    pub fn clear(&self) {
        let mut data = self.data.write().unwrap();
        data.clear();
    }

    /// Get all keys from storage
    pub fn get_keys(&self) -> Vec<String> {
        let data = self.data.read().unwrap();
        data.keys().cloned().collect()
    }

    /// Get length of storage
    pub fn get_length(&self) -> usize {
        let data = self.data.read().unwrap();
        data.len()
    }

    /// Get a key by index
    pub fn get_key(&self, index: usize) -> Option<String> {
        let data = self.data.read().unwrap();
        data.keys().nth(index).cloned()
    }
}
