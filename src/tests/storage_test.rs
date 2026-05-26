use crate::storage::disk_storage::DiskStorage;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
mod storage_test {
    use super::*;

    fn temp_root() -> PathBuf {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        std::env::temp_dir().join(format!("cocos4-rust-storage-suite-{nonce}"))
    }

    #[test]
    fn disk_storage_persists_bytes_to_disk() {
        let path = temp_root();
        let mut storage = DiskStorage::new(path.to_str().unwrap());
        assert!(storage.save("bundle/config.bin", b"cfg"));
        assert_eq!(storage.load("bundle/config.bin"), Some(b"cfg".to_vec()));
        assert!(path.join("bundle/config.bin").exists());
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn disk_storage_tracks_nested_total_size_equivalent_case() {
        let path = temp_root();
        let mut storage = DiskStorage::new(path.to_str().unwrap());
        assert!(storage.save("bundle/a.bin", b"123"));
        assert!(storage.save("bundle/sub/b.bin", b"4567"));
        assert_eq!(storage.get_total_size(), 7);
        let _ = fs::remove_dir_all(path);
    }
}
