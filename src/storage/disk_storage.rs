use std::collections::HashMap;
use std::path::PathBuf;

pub struct DiskStorage {
    base_path: PathBuf,
    cache: HashMap<String, Vec<u8>>,
}

impl DiskStorage {
    pub fn new(base_path: &str) -> Self {
        Self { base_path: PathBuf::from(base_path), cache: HashMap::new() }
    }

    pub fn save(&mut self, key: &str, data: &[u8]) -> bool {
        self.cache.insert(key.to_string(), data.to_vec());
        true
    }

    pub fn load(&self, key: &str) -> Option<Vec<u8>> {
        self.cache.get(key).cloned()
    }

    pub fn remove(&mut self, key: &str) -> bool {
        self.cache.remove(key).is_some()
    }

    pub fn exists(&self, key: &str) -> bool {
        self.cache.contains_key(key)
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn get_size(&self, key: &str) -> Option<usize> {
        self.cache.get(key).map(|d| d.len())
    }

    pub fn get_total_size(&self) -> usize {
        self.cache.values().map(|d| d.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_storage_new() {
        let storage = DiskStorage::new("/tmp/cocos");
        assert_eq!(storage.base_path, PathBuf::from("/tmp/cocos"));
    }

    #[test]
    fn test_save_load() {
        let mut storage = DiskStorage::new("/tmp/cocos");
        storage.save("level1.dat", b"hello world");
        let data = storage.load("level1.dat");
        assert_eq!(data, Some(b"hello world".to_vec()));
    }

    #[test]
    fn test_remove() {
        let mut storage = DiskStorage::new("/tmp/cocos");
        storage.save("temp.dat", b"data");
        assert!(storage.exists("temp.dat"));
        assert!(storage.remove("temp.dat"));
        assert!(!storage.exists("temp.dat"));
    }

    #[test]
    fn test_get_size() {
        let mut storage = DiskStorage::new("/tmp/cocos");
        storage.save("data", b"12345");
        assert_eq!(storage.get_size("data"), Some(5));
    }

    #[test]
    fn test_clear() {
        let mut storage = DiskStorage::new("/tmp/cocos");
        storage.save("a", b"1");
        storage.save("b", b"22");
        storage.clear();
        assert_eq!(storage.get_total_size(), 0);
    }
}
