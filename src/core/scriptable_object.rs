use std::collections::HashMap;
use std::any::{Any, TypeId};
use crate::serialization::value::SerializedValue;

pub trait ScriptableObject: Any + Send + Sync {
    fn type_name(&self) -> &'static str;
    fn serialize(&self) -> SerializedValue;
    fn deserialize(&mut self, data: &SerializedValue);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn on_validate(&mut self) {}
}

pub struct ScriptableObjectRegistry {
    factories: HashMap<String, Box<dyn Fn() -> Box<dyn ScriptableObject> + Send + Sync>>,
}

impl ScriptableObjectRegistry {
    pub fn new() -> Self {
        ScriptableObjectRegistry { factories: HashMap::new() }
    }

    pub fn register<T, F>(&mut self, type_name: &str, factory: F)
    where
        T: ScriptableObject + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.factories.insert(type_name.to_string(), Box::new(move || Box::new(factory())));
    }

    pub fn create(&self, type_name: &str) -> Option<Box<dyn ScriptableObject>> {
        self.factories.get(type_name).map(|f| f())
    }

    pub fn is_registered(&self, type_name: &str) -> bool {
        self.factories.contains_key(type_name)
    }

    pub fn registered_types(&self) -> Vec<&str> {
        let mut v: Vec<&str> = self.factories.keys().map(|s| s.as_str()).collect();
        v.sort();
        v
    }
}

impl Default for ScriptableObjectRegistry {
    fn default() -> Self { Self::new() }
}

pub struct SoDatabase {
    objects: HashMap<String, Box<dyn ScriptableObject>>,
    registry: ScriptableObjectRegistry,
}

impl SoDatabase {
    pub fn new() -> Self {
        SoDatabase {
            objects: HashMap::new(),
            registry: ScriptableObjectRegistry::new(),
        }
    }

    pub fn register_type<T, F>(&mut self, type_name: &str, factory: F)
    where
        T: ScriptableObject + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.registry.register(type_name, factory);
    }

    pub fn create_and_store(&mut self, id: &str, type_name: &str) -> bool {
        if let Some(obj) = self.registry.create(type_name) {
            self.objects.insert(id.to_string(), obj);
            true
        } else {
            false
        }
    }

    pub fn store<T: ScriptableObject + 'static>(&mut self, id: &str, obj: T) {
        self.objects.insert(id.to_string(), Box::new(obj));
    }

    pub fn get<T: ScriptableObject + 'static>(&self, id: &str) -> Option<&T> {
        self.objects.get(id)?.as_any().downcast_ref::<T>()
    }

    pub fn get_mut<T: ScriptableObject + 'static>(&mut self, id: &str) -> Option<&mut T> {
        self.objects.get_mut(id)?.as_any_mut().downcast_mut::<T>()
    }

    pub fn remove(&mut self, id: &str) -> bool {
        self.objects.remove(id).is_some()
    }

    pub fn contains(&self, id: &str) -> bool {
        self.objects.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    pub fn save_to_json(&self, id: &str) -> Option<String> {
        let obj = self.objects.get(id)?;
        Some(obj.serialize().to_json_string())
    }

    pub fn load_from_json(&mut self, id: &str, json: &str) -> bool {
        use crate::serialization::deserializer::Deserializer;
        let data = match Deserializer::from_json(json) {
            Ok(d) => d.get_value().clone(),
            Err(_) => return false,
        };
        if let Some(obj) = self.objects.get_mut(id) {
            obj.deserialize(&data);
            obj.on_validate();
            true
        } else {
            false
        }
    }
}

impl Default for SoDatabase {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug, Clone)]
    struct GameConfig {
        pub difficulty: i64,
        pub music_vol: f64,
        pub player_name: String,
    }

    impl GameConfig {
        fn new() -> Self {
            GameConfig { difficulty: 1, music_vol: 0.8, player_name: "Player".to_string() }
        }
    }

    impl ScriptableObject for GameConfig {
        fn type_name(&self) -> &'static str { "GameConfig" }
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }

        fn serialize(&self) -> SerializedValue {
            let mut map = HashMap::new();
            map.insert("difficulty".to_string(), SerializedValue::Int(self.difficulty));
            map.insert("music_vol".to_string(), SerializedValue::Float(self.music_vol));
            map.insert("player_name".to_string(), SerializedValue::String(self.player_name.clone()));
            SerializedValue::Object(map)
        }

        fn deserialize(&mut self, data: &SerializedValue) {
            if let Some(v) = data.get("difficulty").and_then(|v| v.as_int()) {
                self.difficulty = v;
            }
            if let Some(v) = data.get("music_vol").and_then(|v| v.as_f64()) {
                self.music_vol = v;
            }
            if let Some(v) = data.get("player_name").and_then(|v| v.as_str()) {
                self.player_name = v.to_string();
            }
        }

        fn on_validate(&mut self) {
            self.difficulty = self.difficulty.clamp(1, 5);
            self.music_vol = self.music_vol.clamp(0.0, 1.0);
        }
    }

    #[derive(Debug, Clone)]
    struct LevelData {
        pub level: i64,
        pub enemies: i64,
    }

    impl LevelData {
        fn new() -> Self { LevelData { level: 1, enemies: 10 } }
    }

    impl ScriptableObject for LevelData {
        fn type_name(&self) -> &'static str { "LevelData" }
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }
        fn serialize(&self) -> SerializedValue {
            let mut map = HashMap::new();
            map.insert("level".to_string(), SerializedValue::Int(self.level));
            map.insert("enemies".to_string(), SerializedValue::Int(self.enemies));
            SerializedValue::Object(map)
        }
        fn deserialize(&mut self, data: &SerializedValue) {
            if let Some(v) = data.get("level").and_then(|v| v.as_int()) { self.level = v; }
            if let Some(v) = data.get("enemies").and_then(|v| v.as_int()) { self.enemies = v; }
        }
    }

    #[test]
    fn test_registry_register_and_create() {
        let mut reg = ScriptableObjectRegistry::new();
        reg.register::<GameConfig, _>("GameConfig", GameConfig::new);
        assert!(reg.is_registered("GameConfig"));
        let obj = reg.create("GameConfig");
        assert!(obj.is_some());
        assert_eq!(obj.unwrap().type_name(), "GameConfig");
    }

    #[test]
    fn test_registry_unknown_returns_none() {
        let reg = ScriptableObjectRegistry::new();
        assert!(reg.create("Unknown").is_none());
    }

    #[test]
    fn test_registry_sorted_types() {
        let mut reg = ScriptableObjectRegistry::new();
        reg.register::<LevelData, _>("LevelData", LevelData::new);
        reg.register::<GameConfig, _>("GameConfig", GameConfig::new);
        let types = reg.registered_types();
        assert_eq!(types, vec!["GameConfig", "LevelData"]);
    }

    #[test]
    fn test_database_store_and_get() {
        let mut db = SoDatabase::new();
        db.store("cfg", GameConfig::new());
        assert!(db.contains("cfg"));
        let cfg = db.get::<GameConfig>("cfg");
        assert!(cfg.is_some());
        assert_eq!(cfg.unwrap().difficulty, 1);
    }

    #[test]
    fn test_database_get_mut() {
        let mut db = SoDatabase::new();
        db.store("cfg", GameConfig::new());
        db.get_mut::<GameConfig>("cfg").unwrap().difficulty = 3;
        assert_eq!(db.get::<GameConfig>("cfg").unwrap().difficulty, 3);
    }

    #[test]
    fn test_database_remove() {
        let mut db = SoDatabase::new();
        db.store("cfg", GameConfig::new());
        assert!(db.remove("cfg"));
        assert!(!db.contains("cfg"));
        assert!(!db.remove("cfg"));
    }

    #[test]
    fn test_database_create_and_store() {
        let mut db = SoDatabase::new();
        db.register_type::<GameConfig, _>("GameConfig", GameConfig::new);
        assert!(db.create_and_store("cfg", "GameConfig"));
        assert!(db.contains("cfg"));
        assert!(!db.create_and_store("x", "Unknown"));
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let mut db = SoDatabase::new();
        db.store("cfg", GameConfig { difficulty: 3, music_vol: 0.5, player_name: "Hero".to_string() });
        let json = db.save_to_json("cfg").unwrap();
        db.store("cfg2", GameConfig::new());
        let ok = db.load_from_json("cfg2", &json);
        assert!(ok);
        let cfg2 = db.get::<GameConfig>("cfg2").unwrap();
        assert_eq!(cfg2.difficulty, 3);
        assert!((cfg2.music_vol - 0.5).abs() < 1e-9);
        assert_eq!(cfg2.player_name, "Hero");
    }

    #[test]
    fn test_on_validate_clamps_values() {
        let mut db = SoDatabase::new();
        db.store("cfg", GameConfig::new());
        let bad_json = r#"{"difficulty":99,"music_vol":2.0,"player_name":"Test"}"#;
        db.load_from_json("cfg", bad_json);
        let cfg = db.get::<GameConfig>("cfg").unwrap();
        assert_eq!(cfg.difficulty, 5);
        assert!((cfg.music_vol - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_database_len_is_empty() {
        let mut db = SoDatabase::new();
        assert!(db.is_empty());
        db.store("a", GameConfig::new());
        db.store("b", LevelData::new());
        assert_eq!(db.len(), 2);
    }

    #[test]
    fn test_multiple_types_in_db() {
        let mut db = SoDatabase::new();
        db.store("config", GameConfig::new());
        db.store("level1", LevelData { level: 1, enemies: 5 });
        let cfg = db.get::<GameConfig>("config");
        let lvl = db.get::<LevelData>("level1");
        assert!(cfg.is_some());
        assert!(lvl.is_some());
        assert_eq!(lvl.unwrap().enemies, 5);
    }
}
