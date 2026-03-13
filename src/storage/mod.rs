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

    pub fn get_item(&self, key: &str) -> Option<String> {
        let data = self.data.read().unwrap();
        data.get(key).cloned()
    }

    pub fn set_item(&self, key: &str, value: &str) {
        let mut data = self.data.write().unwrap();
        data.insert(key.to_string(), value.to_string());
    }

    pub fn remove_item(&self, key: &str) {
        let mut data = self.data.write().unwrap();
        data.remove(key);
    }

    pub fn clear(&self) {
        let mut data = self.data.write().unwrap();
        data.clear();
    }

    pub fn get_keys(&self) -> Vec<String> {
        let data = self.data.read().unwrap();
        data.keys().cloned().collect()
    }

    pub fn get_length(&self) -> usize {
        let data = self.data.read().unwrap();
        data.len()
    }

    pub fn get_key(&self, index: usize) -> Option<String> {
        let data = self.data.read().unwrap();
        data.keys().nth(index).cloned()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        let data = self.data.read().unwrap();
        data.contains_key(key)
    }
}

impl Default for LocalStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for LocalStorage {
    fn clone(&self) -> Self {
        LocalStorage {
            data: Arc::clone(&self.data),
        }
    }
}

/// Simple JSON-like value type for storage
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl JsonValue {
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let JsonValue::Bool(v) = self { Some(*v) } else { None }
    }

    pub fn as_i64(&self) -> Option<i64> {
        if let JsonValue::Int(v) = self { Some(*v) } else { None }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            JsonValue::Float(v) => Some(*v),
            JsonValue::Int(v) => Some(*v as f64),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let JsonValue::String(v) = self { Some(v.as_str()) } else { None }
    }

    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        if let JsonValue::Array(v) = self { Some(v) } else { None }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, JsonValue>> {
        if let JsonValue::Object(v) = self { Some(v) } else { None }
    }

    pub fn to_json_string(&self) -> String {
        match self {
            JsonValue::Null => "null".to_string(),
            JsonValue::Bool(v) => v.to_string(),
            JsonValue::Int(v) => v.to_string(),
            JsonValue::Float(v) => format!("{}", v),
            JsonValue::String(v) => format!("\"{}\"", v.replace('\\', "\\\\").replace('"', "\\\"")),
            JsonValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_json_string()).collect();
                format!("[{}]", items.join(","))
            }
            JsonValue::Object(obj) => {
                let items: Vec<String> = obj.iter()
                    .map(|(k, v)| format!("\"{}\":{}", k, v.to_json_string()))
                    .collect();
                format!("{{{}}}", items.join(","))
            }
        }
    }

    pub fn parse(s: &str) -> Result<JsonValue, String> {
        let s = s.trim();
        if s == "null" {
            return Ok(JsonValue::Null);
        }
        if s == "true" {
            return Ok(JsonValue::Bool(true));
        }
        if s == "false" {
            return Ok(JsonValue::Bool(false));
        }
        if s.starts_with('"') && s.ends_with('"') {
            let inner = &s[1..s.len() - 1];
            return Ok(JsonValue::String(inner.replace("\\\"", "\"").replace("\\\\", "\\")));
        }
        if s.starts_with('{') && s.ends_with('}') {
            return Ok(JsonValue::Object(HashMap::new()));
        }
        if s.starts_with('[') && s.ends_with(']') {
            return Ok(JsonValue::Array(Vec::new()));
        }
        if s.contains('.') {
            if let Ok(v) = s.parse::<f64>() {
                return Ok(JsonValue::Float(v));
            }
        }
        if let Ok(v) = s.parse::<i64>() {
            return Ok(JsonValue::Int(v));
        }
        Err(format!("Cannot parse JSON: {}", s))
    }
}

impl From<bool> for JsonValue {
    fn from(v: bool) -> Self { JsonValue::Bool(v) }
}

impl From<i32> for JsonValue {
    fn from(v: i32) -> Self { JsonValue::Int(v as i64) }
}

impl From<i64> for JsonValue {
    fn from(v: i64) -> Self { JsonValue::Int(v) }
}

impl From<f32> for JsonValue {
    fn from(v: f32) -> Self { JsonValue::Float(v as f64) }
}

impl From<f64> for JsonValue {
    fn from(v: f64) -> Self { JsonValue::Float(v) }
}

impl From<String> for JsonValue {
    fn from(v: String) -> Self { JsonValue::String(v) }
}

impl From<&str> for JsonValue {
    fn from(v: &str) -> Self { JsonValue::String(v.to_string()) }
}

/// JSON-aware storage that serializes/deserializes values
pub struct JsonStorage {
    local: LocalStorage,
}

impl JsonStorage {
    pub fn new() -> Self {
        JsonStorage {
            local: LocalStorage::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<JsonValue> {
        self.local.get_item(key)
            .and_then(|s| JsonValue::parse(&s).ok())
    }

    pub fn set(&self, key: &str, value: &JsonValue) {
        self.local.set_item(key, &value.to_json_string());
    }

    pub fn remove(&self, key: &str) {
        self.local.remove_item(key);
    }

    pub fn clear(&self) {
        self.local.clear();
    }

    pub fn get_bool(&self, key: &str, default: bool) -> bool {
        self.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
    }

    pub fn get_int(&self, key: &str, default: i64) -> i64 {
        self.get(key).and_then(|v| v.as_i64()).unwrap_or(default)
    }

    pub fn get_float(&self, key: &str, default: f64) -> f64 {
        self.get(key).and_then(|v| v.as_f64()).unwrap_or(default)
    }

    pub fn get_string(&self, key: &str, default: &str) -> String {
        self.get(key)
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| default.to_string())
    }

    pub fn set_bool(&self, key: &str, value: bool) {
        self.set(key, &JsonValue::Bool(value));
    }

    pub fn set_int(&self, key: &str, value: i64) {
        self.set(key, &JsonValue::Int(value));
    }

    pub fn set_float(&self, key: &str, value: f64) {
        self.set(key, &JsonValue::Float(value));
    }

    pub fn set_string(&self, key: &str, value: &str) {
        self.set(key, &JsonValue::String(value.to_string()));
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.local.contains_key(key)
    }

    pub fn get_length(&self) -> usize {
        self.local.get_length()
    }
}

impl Default for JsonStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_storage_basic() {
        let storage = LocalStorage::new();
        storage.set_item("key1", "value1");
        assert_eq!(storage.get_item("key1"), Some("value1".to_string()));
        assert_eq!(storage.get_item("nonexistent"), None);
    }

    #[test]
    fn test_local_storage_remove() {
        let storage = LocalStorage::new();
        storage.set_item("key1", "value1");
        storage.remove_item("key1");
        assert_eq!(storage.get_item("key1"), None);
    }

    #[test]
    fn test_local_storage_clear() {
        let storage = LocalStorage::new();
        storage.set_item("k1", "v1");
        storage.set_item("k2", "v2");
        storage.clear();
        assert_eq!(storage.get_length(), 0);
    }

    #[test]
    fn test_local_storage_keys() {
        let storage = LocalStorage::new();
        storage.set_item("a", "1");
        storage.set_item("b", "2");
        let keys = storage.get_keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"a".to_string()));
        assert!(keys.contains(&"b".to_string()));
    }

    #[test]
    fn test_local_storage_contains_key() {
        let storage = LocalStorage::new();
        storage.set_item("hello", "world");
        assert!(storage.contains_key("hello"));
        assert!(!storage.contains_key("unknown"));
    }

    #[test]
    fn test_json_value_serialize() {
        assert_eq!(JsonValue::Null.to_json_string(), "null");
        assert_eq!(JsonValue::Bool(true).to_json_string(), "true");
        assert_eq!(JsonValue::Bool(false).to_json_string(), "false");
        assert_eq!(JsonValue::Int(42).to_json_string(), "42");
        assert_eq!(JsonValue::String("hello".to_string()).to_json_string(), "\"hello\"");
    }

    #[test]
    fn test_json_value_parse() {
        assert_eq!(JsonValue::parse("null").unwrap(), JsonValue::Null);
        assert_eq!(JsonValue::parse("true").unwrap(), JsonValue::Bool(true));
        assert_eq!(JsonValue::parse("false").unwrap(), JsonValue::Bool(false));
        assert_eq!(JsonValue::parse("42").unwrap(), JsonValue::Int(42));
        assert_eq!(JsonValue::parse("3.14").unwrap(), JsonValue::Float(3.14));
        assert_eq!(JsonValue::parse("\"hello\"").unwrap(), JsonValue::String("hello".to_string()));
    }

    #[test]
    fn test_json_storage_bool() {
        let storage = JsonStorage::new();
        storage.set_bool("flag", true);
        assert_eq!(storage.get_bool("flag", false), true);
        assert_eq!(storage.get_bool("missing", false), false);
    }

    #[test]
    fn test_json_storage_int() {
        let storage = JsonStorage::new();
        storage.set_int("count", 100);
        assert_eq!(storage.get_int("count", 0), 100);
    }

    #[test]
    fn test_json_storage_float() {
        let storage = JsonStorage::new();
        storage.set_float("ratio", 0.75);
        let v = storage.get_float("ratio", 0.0);
        assert!((v - 0.75).abs() < 1e-9);
    }

    #[test]
    fn test_json_storage_string() {
        let storage = JsonStorage::new();
        storage.set_string("name", "player1");
        assert_eq!(storage.get_string("name", ""), "player1");
    }

    #[test]
    fn test_json_value_from_conversions() {
        let v: JsonValue = true.into();
        assert_eq!(v, JsonValue::Bool(true));
        let v: JsonValue = 42i32.into();
        assert_eq!(v, JsonValue::Int(42));
        let v: JsonValue = "hello".into();
        assert_eq!(v, JsonValue::String("hello".to_string()));
    }
}
