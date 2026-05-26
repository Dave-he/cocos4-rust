use std::fs;
use std::path::{Path, PathBuf};

pub struct DiskStorage {
    base_path: PathBuf,
}

impl DiskStorage {
    pub fn new(base_path: &str) -> Self {
        let path = PathBuf::from(base_path);
        let _ = fs::create_dir_all(&path);
        Self { base_path: path }
    }

    pub fn save(&mut self, key: &str, data: &[u8]) -> bool {
        let path = self.resolve_path(key);
        if let Some(parent) = path.parent() {
            if fs::create_dir_all(parent).is_err() {
                return false;
            }
        }
        fs::write(path, data).is_ok()
    }

    pub fn load(&self, key: &str) -> Option<Vec<u8>> {
        fs::read(self.resolve_path(key)).ok()
    }

    pub fn remove(&mut self, key: &str) -> bool {
        let path = self.resolve_path(key);
        !path.exists() || fs::remove_file(path).is_ok()
    }

    pub fn exists(&self, key: &str) -> bool {
        self.resolve_path(key).exists()
    }

    pub fn clear(&mut self) {
        let _ = fs::remove_dir_all(&self.base_path);
        let _ = fs::create_dir_all(&self.base_path);
    }

    pub fn get_size(&self, key: &str) -> Option<usize> {
        fs::metadata(self.resolve_path(key)).ok().map(|m| m.len() as usize)
    }

    pub fn get_total_size(&self) -> usize {
        dir_size(&self.base_path)
    }

    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    fn resolve_path(&self, key: &str) -> PathBuf {
        let sanitized = key.replace("..", "_");
        self.base_path.join(sanitized)
    }
}

fn dir_size(path: &Path) -> usize {
    let mut total = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total += metadata.len() as usize;
                } else if metadata.is_dir() {
                    total += dir_size(&entry_path);
                }
            }
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("cocos4-rust-storage-{nonce}"))
    }

    #[test]
    fn test_disk_storage_new() {
        let path = temp_root();
        let storage = DiskStorage::new(path.to_str().unwrap());
        assert_eq!(storage.base_path(), path.as_path());
        assert!(storage.base_path().exists());
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_save_load() {
        let path = temp_root();
        let mut storage = DiskStorage::new(path.to_str().unwrap());
        assert!(storage.save("level1.dat", b"hello world"));
        let data = storage.load("level1.dat");
        assert_eq!(data, Some(b"hello world".to_vec()));
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_remove() {
        let path = temp_root();
        let mut storage = DiskStorage::new(path.to_str().unwrap());
        storage.save("temp.dat", b"data");
        assert!(storage.exists("temp.dat"));
        assert!(storage.remove("temp.dat"));
        assert!(!storage.exists("temp.dat"));
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_get_size() {
        let path = temp_root();
        let mut storage = DiskStorage::new(path.to_str().unwrap());
        storage.save("data", b"12345");
        assert_eq!(storage.get_size("data"), Some(5));
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_clear() {
        let path = temp_root();
        let mut storage = DiskStorage::new(path.to_str().unwrap());
        storage.save("a", b"1");
        storage.save("nested/b", b"22");
        storage.clear();
        assert_eq!(storage.get_total_size(), 0);
        assert!(storage.base_path().exists());
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_remove_missing_is_idempotent_equivalent_case() {
        let path = temp_root();
        let mut storage = DiskStorage::new(path.to_str().unwrap());
        assert!(storage.remove("missing.bin"));
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_sanitize_parent_segments_equivalent_case() {
        let path = temp_root();
        let mut storage = DiskStorage::new(path.to_str().unwrap());
        assert!(storage.save("../unsafe.bin", b"safe"));
        assert!(path.join("_/unsafe.bin").exists());
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_clear_then_resave_equivalent_case() {
        let path = temp_root();
        let mut storage = DiskStorage::new(path.to_str().unwrap());
        assert!(storage.save("a.bin", b"a"));
        storage.clear();
        assert!(storage.save("b.bin", b"bb"));
        assert_eq!(storage.load("b.bin"), Some(b"bb".to_vec()));
        let _ = fs::remove_dir_all(path);
    }
}
