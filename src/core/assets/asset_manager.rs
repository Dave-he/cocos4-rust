use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::any::{Any, TypeId};

pub trait Asset: Any + Send + Sync {
    fn get_uuid(&self) -> &str;
    fn get_name(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadState {
    Unloaded,
    Loading,
    Loaded,
    Failed,
}

struct AssetEntry {
    uuid: String,
    name: String,
    load_state: LoadState,
    ref_count: u32,
    asset: Option<Box<dyn Asset>>,
    type_id: TypeId,
}

impl AssetEntry {
    fn new(uuid: &str, name: &str, type_id: TypeId) -> Self {
        AssetEntry {
            uuid: uuid.to_string(),
            name: name.to_string(),
            load_state: LoadState::Unloaded,
            ref_count: 0,
            asset: None,
            type_id,
        }
    }
}

pub type AssetPtr<T> = Arc<T>;
type AssetLoader = Box<dyn Fn(&str) -> Option<Box<dyn Asset>> + Send + Sync>;

pub struct AssetManager {
    entries: HashMap<String, AssetEntry>,
    loaders: HashMap<TypeId, AssetLoader>,
    uuid_by_name: HashMap<String, String>,
    total_loaded: u64,
    total_failed: u64,
}

impl AssetManager {
    pub fn new() -> Self {
        AssetManager {
            entries: HashMap::new(),
            loaders: HashMap::new(),
            uuid_by_name: HashMap::new(),
            total_loaded: 0,
            total_failed: 0,
        }
    }

    pub fn register_loader<T: Asset + 'static, F>(&mut self, loader: F)
    where
        F: Fn(&str) -> Option<Box<dyn Asset>> + Send + Sync + 'static,
    {
        self.loaders.insert(TypeId::of::<T>(), Box::new(loader));
    }

    pub fn add_asset<T: Asset + 'static>(&mut self, uuid: &str, name: &str) {
        let entry = AssetEntry::new(uuid, name, TypeId::of::<T>());
        self.entries.insert(uuid.to_string(), entry);
        self.uuid_by_name.insert(name.to_string(), uuid.to_string());
    }

    pub fn load<T: Asset + 'static>(&mut self, uuid: &str) -> bool {
        let type_id = TypeId::of::<T>();
        if !self.entries.contains_key(uuid) {
            let entry = AssetEntry::new(uuid, uuid, type_id);
            self.entries.insert(uuid.to_string(), entry);
        }
        let entry = match self.entries.get_mut(uuid) {
            Some(e) => e,
            None => return false,
        };
        if entry.load_state == LoadState::Loaded {
            entry.ref_count += 1;
            return true;
        }
        entry.load_state = LoadState::Loading;
        let loader = self.loaders.get(&type_id);
        match loader {
            Some(loader) => {
                let result = loader(uuid);
                let entry = self.entries.get_mut(uuid).unwrap();
                match result {
                    Some(asset) => {
                        entry.asset = Some(asset);
                        entry.load_state = LoadState::Loaded;
                        entry.ref_count += 1;
                        self.total_loaded += 1;
                        true
                    }
                    None => {
                        entry.load_state = LoadState::Failed;
                        self.total_failed += 1;
                        false
                    }
                }
            }
            None => {
                if let Some(entry) = self.entries.get_mut(uuid) {
                    entry.load_state = LoadState::Failed;
                    self.total_failed += 1;
                }
                false
            }
        }
    }

    pub fn get<T: Asset + 'static>(&self, uuid: &str) -> Option<&T> {
        let entry = self.entries.get(uuid)?;
        if entry.load_state != LoadState::Loaded {
            return None;
        }
        entry.asset.as_ref()?.as_any().downcast_ref::<T>()
    }

    pub fn get_by_name<T: Asset + 'static>(&self, name: &str) -> Option<&T> {
        let uuid = self.uuid_by_name.get(name)?;
        self.get::<T>(uuid)
    }

    pub fn retain(&mut self, uuid: &str) {
        if let Some(entry) = self.entries.get_mut(uuid) {
            entry.ref_count = entry.ref_count.saturating_add(1);
        }
    }

    pub fn release(&mut self, uuid: &str) {
        if let Some(entry) = self.entries.get_mut(uuid) {
            entry.ref_count = entry.ref_count.saturating_sub(1);
            if entry.ref_count == 0 {
                entry.asset = None;
                entry.load_state = LoadState::Unloaded;
            }
        }
    }

    pub fn release_all(&mut self) {
        for entry in self.entries.values_mut() {
            entry.ref_count = 0;
            entry.asset = None;
            entry.load_state = LoadState::Unloaded;
        }
    }

    pub fn get_load_state(&self, uuid: &str) -> Option<LoadState> {
        self.entries.get(uuid).map(|e| e.load_state)
    }

    pub fn get_ref_count(&self, uuid: &str) -> u32 {
        self.entries.get(uuid).map(|e| e.ref_count).unwrap_or(0)
    }

    pub fn is_loaded(&self, uuid: &str) -> bool {
        self.entries.get(uuid).map(|e| e.load_state == LoadState::Loaded).unwrap_or(false)
    }

    pub fn get_total_loaded(&self) -> u64 {
        self.total_loaded
    }

    pub fn get_total_failed(&self) -> u64 {
        self.total_failed
    }

    pub fn get_loaded_count(&self) -> usize {
        self.entries.values().filter(|e| e.load_state == LoadState::Loaded).count()
    }

    pub fn get_registered_count(&self) -> usize {
        self.entries.len()
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TextAsset {
        uuid: String,
        name: String,
        pub content: String,
    }

    impl Asset for TextAsset {
        fn get_uuid(&self) -> &str { &self.uuid }
        fn get_name(&self) -> &str { &self.name }
        fn as_any(&self) -> &dyn Any { self }
    }

    fn make_manager() -> AssetManager {
        let mut am = AssetManager::new();
        let loader = |uuid: &str| -> Option<Box<dyn Asset>> {
            Some(Box::new(TextAsset {
                uuid: uuid.to_string(),
                name: uuid.to_string(),
                content: format!("content of {}", uuid),
            }))
        };
        am.register_loader::<TextAsset, _>(loader);
        am
    }

    #[test]
    fn test_asset_manager_new() {
        let am = AssetManager::new();
        assert_eq!(am.get_loaded_count(), 0);
        assert_eq!(am.get_registered_count(), 0);
    }

    #[test]
    fn test_load_asset() {
        let mut am = make_manager();
        am.add_asset::<TextAsset>("uuid-1", "text1");
        let ok = am.load::<TextAsset>("uuid-1");
        assert!(ok);
        assert!(am.is_loaded("uuid-1"));
        assert_eq!(am.get_load_state("uuid-1"), Some(LoadState::Loaded));
        assert_eq!(am.get_total_loaded(), 1);
    }

    #[test]
    fn test_get_asset() {
        let mut am = make_manager();
        am.add_asset::<TextAsset>("uuid-2", "text2");
        am.load::<TextAsset>("uuid-2");
        let asset = am.get::<TextAsset>("uuid-2");
        assert!(asset.is_some());
        assert_eq!(asset.unwrap().content, "content of uuid-2");
    }

    #[test]
    fn test_get_by_name() {
        let mut am = make_manager();
        am.add_asset::<TextAsset>("uuid-3", "mytext");
        am.load::<TextAsset>("uuid-3");
        let asset = am.get_by_name::<TextAsset>("mytext");
        assert!(asset.is_some());
    }

    #[test]
    fn test_load_no_loader_fails() {
        let mut am = AssetManager::new();
        am.add_asset::<TextAsset>("uuid-4", "t4");
        let ok = am.load::<TextAsset>("uuid-4");
        assert!(!ok);
        assert_eq!(am.get_load_state("uuid-4"), Some(LoadState::Failed));
        assert_eq!(am.get_total_failed(), 1);
    }

    #[test]
    fn test_ref_counting() {
        let mut am = make_manager();
        am.add_asset::<TextAsset>("uuid-5", "t5");
        am.load::<TextAsset>("uuid-5");
        assert_eq!(am.get_ref_count("uuid-5"), 1);
        am.retain("uuid-5");
        assert_eq!(am.get_ref_count("uuid-5"), 2);
        am.release("uuid-5");
        assert_eq!(am.get_ref_count("uuid-5"), 1);
        assert!(am.is_loaded("uuid-5"));
        am.release("uuid-5");
        assert_eq!(am.get_ref_count("uuid-5"), 0);
        assert!(!am.is_loaded("uuid-5"));
    }

    #[test]
    fn test_load_twice_increments_ref() {
        let mut am = make_manager();
        am.add_asset::<TextAsset>("uuid-6", "t6");
        am.load::<TextAsset>("uuid-6");
        am.load::<TextAsset>("uuid-6");
        assert_eq!(am.get_ref_count("uuid-6"), 2);
    }

    #[test]
    fn test_release_all() {
        let mut am = make_manager();
        am.add_asset::<TextAsset>("uuid-7", "t7");
        am.add_asset::<TextAsset>("uuid-8", "t8");
        am.load::<TextAsset>("uuid-7");
        am.load::<TextAsset>("uuid-8");
        assert_eq!(am.get_loaded_count(), 2);
        am.release_all();
        assert_eq!(am.get_loaded_count(), 0);
    }

    #[test]
    fn test_get_unloaded_returns_none() {
        let mut am = make_manager();
        am.add_asset::<TextAsset>("uuid-9", "t9");
        let asset = am.get::<TextAsset>("uuid-9");
        assert!(asset.is_none());
    }

    #[test]
    fn test_load_unknown_uuid_auto_entry() {
        let mut am = make_manager();
        let ok = am.load::<TextAsset>("auto-uuid");
        assert!(ok);
        assert!(am.is_loaded("auto-uuid"));
    }
}
