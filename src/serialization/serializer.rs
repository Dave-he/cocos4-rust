use crate::serialization::value::SerializedValue;
use std::collections::HashMap;

pub struct Serializer {
    data: SerializedValue,
}

impl Serializer {
    pub fn new() -> Self {
        Serializer {
            data: SerializedValue::Object(HashMap::new()),
        }
    }

    pub fn write_bool(&mut self, key: &str, value: bool) {
        self.set(key, SerializedValue::Bool(value));
    }

    pub fn write_int(&mut self, key: &str, value: i64) {
        self.set(key, SerializedValue::Int(value));
    }

    pub fn write_float(&mut self, key: &str, value: f64) {
        self.set(key, SerializedValue::Float(value));
    }

    pub fn write_str(&mut self, key: &str, value: &str) {
        self.set(key, SerializedValue::String(value.to_string()));
    }

    pub fn write_null(&mut self, key: &str) {
        self.set(key, SerializedValue::Null);
    }

    pub fn write_array(&mut self, key: &str, values: Vec<SerializedValue>) {
        self.set(key, SerializedValue::Array(values));
    }

    pub fn write_object(&mut self, key: &str, obj: HashMap<String, SerializedValue>) {
        self.set(key, SerializedValue::Object(obj));
    }

    pub fn write_nested(&mut self, key: &str, nested: Serializer) {
        self.set(key, nested.finish());
    }

    pub fn write_value(&mut self, key: &str, value: SerializedValue) {
        self.set(key, value);
    }

    fn set(&mut self, key: &str, value: SerializedValue) {
        if let SerializedValue::Object(ref mut map) = self.data {
            map.insert(key.to_string(), value);
        }
    }

    pub fn finish(self) -> SerializedValue {
        self.data
    }

    pub fn to_json(&self) -> String {
        self.data.to_json_string()
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        use crate::serialization::deserializer::Deserializer;
        let d = Deserializer::from_json(json)?;
        Ok(Serializer {
            data: d.get_value().clone(),
        })
    }
}

impl Default for Serializer {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Serializer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Serializer({})", self.data.to_json_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serializer_basic() {
        let mut s = Serializer::new();
        s.write_str("name", "Player");
        s.write_int("score", 100);
        s.write_float("health", 0.75);
        s.write_bool("alive", true);
        let v = s.finish();
        assert_eq!(v.get("name").unwrap().as_str(), Some("Player"));
        assert_eq!(v.get("score").unwrap().as_int(), Some(100));
        assert!(v.get("alive").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_serializer_to_json() {
        let mut s = Serializer::new();
        s.write_int("x", 1);
        let json = s.to_json();
        assert!(json.contains("\"x\""));
        assert!(json.contains('1'));
    }

    #[test]
    fn test_serializer_array() {
        let mut s = Serializer::new();
        s.write_array(
            "items",
            vec![
                SerializedValue::from(1i64),
                SerializedValue::from(2i64),
                SerializedValue::from(3i64),
            ],
        );
        let v = s.finish();
        let arr = v.get("items").unwrap().as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_serializer_null() {
        let mut s = Serializer::new();
        s.write_null("empty");
        let v = s.finish();
        assert!(v.get("empty").unwrap().is_null());
    }

    #[test]
    fn test_serializer_nested() {
        let mut outer = Serializer::new();
        outer.write_str("type", "enemy");
        let mut inner = Serializer::new();
        inner.write_float("x", 1.5);
        inner.write_float("y", 2.5);
        outer.write_nested("position", inner);
        let v = outer.finish();
        let pos = v.get("position").unwrap();
        assert!((pos.get("x").unwrap().as_f64().unwrap() - 1.5).abs() < 1e-9);
    }

    #[test]
    fn test_serializer_write_value() {
        let mut s = Serializer::new();
        s.write_value("v", SerializedValue::from(42i64));
        let v = s.finish();
        assert_eq!(v.get("v").unwrap().as_int(), Some(42));
    }

    #[test]
    fn test_serializer_from_json_roundtrip() {
        let mut s = Serializer::new();
        s.write_str("name", "hero");
        s.write_int("level", 5);
        let json = s.to_json();
        let s2 = Serializer::from_json(&json).unwrap();
        let v = s2.finish();
        assert_eq!(v.get("name").unwrap().as_str(), Some("hero"));
        assert_eq!(v.get("level").unwrap().as_int(), Some(5));
    }

    #[test]
    fn test_serializer_deeply_nested() {
        let mut s = Serializer::new();
        let mut stats = Serializer::new();
        stats.write_int("hp", 100);
        stats.write_int("mp", 50);
        let mut position = Serializer::new();
        position.write_float("x", 10.0);
        position.write_float("y", 20.0);
        stats.write_nested("position", position);
        s.write_nested("stats", stats);
        let json = s.to_json();
        let d = Serializer::from_json(&json).unwrap();
        let v = d.finish();
        let st = v.get("stats").unwrap();
        assert_eq!(st.get("hp").unwrap().as_int(), Some(100));
        let pos = st.get("position").unwrap();
        assert!((pos.get("x").unwrap().as_f64().unwrap() - 10.0).abs() < 1e-9);
    }

    #[test]
    fn test_serializer_primitives_equivalent_case() {
        let mut s = Serializer::new();
        s.write_null("none");
        s.write_bool("flag", true);
        s.write_int("lives", 3);
        s.write_float("speed", 2.5);
        s.write_str("name", "hero");

        let json = s.to_json();
        let restored = Serializer::from_json(&json).unwrap().finish();

        assert!(restored.get("none").unwrap().is_null());
        assert_eq!(restored.get("flag").unwrap().as_bool(), Some(true));
        assert_eq!(restored.get("lives").unwrap().as_int(), Some(3));
        assert_eq!(restored.get("speed").unwrap().as_f64(), Some(2.5));
        assert_eq!(restored.get("name").unwrap().as_str(), Some("hero"));
    }

    #[test]
    fn test_serializer_dictionary_equivalent_case() {
        let mut dictionary = HashMap::new();
        dictionary.insert("hp".to_string(), SerializedValue::from(120i64));
        dictionary.insert("mana".to_string(), SerializedValue::from(45i64));

        let mut nested = HashMap::new();
        nested.insert("stats".to_string(), SerializedValue::Object(dictionary));
        nested.insert("title".to_string(), SerializedValue::from("mage"));

        let mut s = Serializer::new();
        s.write_object("player", nested);
        let json = s.to_json();
        let restored = Serializer::from_json(&json).unwrap().finish();

        let player = restored.get("player").unwrap();
        let stats = player.get("stats").unwrap();
        assert_eq!(player.get("title").unwrap().as_str(), Some("mage"));
        assert_eq!(stats.get("hp").unwrap().as_int(), Some(120));
        assert_eq!(stats.get("mana").unwrap().as_int(), Some(45));
    }
}
