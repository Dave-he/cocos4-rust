use crate::serialization::value::SerializedValue;

pub struct Deserializer {
    data: SerializedValue,
}

impl Deserializer {
    pub fn new(data: SerializedValue) -> Self {
        Deserializer { data }
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        let value = parse_json(json)?;
        Ok(Deserializer { data: value })
    }

    pub fn read_bool(&self, key: &str) -> Option<bool> {
        self.data.get(key)?.as_bool()
    }

    pub fn read_int(&self, key: &str) -> Option<i64> {
        self.data.get(key)?.as_int()
    }

    pub fn read_float(&self, key: &str) -> Option<f64> {
        self.data.get(key)?.as_f64()
    }

    pub fn read_str(&self, key: &str) -> Option<&str> {
        self.data.get(key)?.as_str()
    }

    pub fn read_array(&self, key: &str) -> Option<&Vec<SerializedValue>> {
        self.data.get(key)?.as_array()
    }

    pub fn is_null(&self, key: &str) -> bool {
        self.data.get(key).map(|v| v.is_null()).unwrap_or(true)
    }

    pub fn has_key(&self, key: &str) -> bool {
        self.data.get(key).is_some()
    }

    pub fn get_value(&self) -> &SerializedValue {
        &self.data
    }

    pub fn get_nested(&self, key: &str) -> Option<Deserializer> {
        let child = self.data.get(key)?;
        Some(Deserializer { data: child.clone() })
    }
}

fn parse_json(json: &str) -> Result<SerializedValue, String> {
    let json = json.trim();
    if json == "null" {
        return Ok(SerializedValue::Null);
    }
    if json == "true" {
        return Ok(SerializedValue::Bool(true));
    }
    if json == "false" {
        return Ok(SerializedValue::Bool(false));
    }
    if json.starts_with('"') && json.ends_with('"') {
        let inner = &json[1..json.len() - 1];
        return Ok(SerializedValue::String(inner.replace("\\\"", "\"")));
    }
    if json.starts_with('[') {
        return parse_json_array(json);
    }
    if json.starts_with('{') {
        return parse_json_object(json);
    }
    if let Ok(i) = json.parse::<i64>() {
        return Ok(SerializedValue::Int(i));
    }
    if let Ok(f) = json.parse::<f64>() {
        return Ok(SerializedValue::Float(f));
    }
    Err(format!("Cannot parse JSON: {}", &json[..json.len().min(40)]))
}

fn parse_json_array(json: &str) -> Result<SerializedValue, String> {
    let inner = json.trim_start_matches('[').trim_end_matches(']');
    if inner.trim().is_empty() {
        return Ok(SerializedValue::Array(vec![]));
    }
    let parts = split_json_values(inner);
    let mut arr = Vec::new();
    for part in parts {
        arr.push(parse_json(part.trim())?);
    }
    Ok(SerializedValue::Array(arr))
}

fn parse_json_object(json: &str) -> Result<SerializedValue, String> {
    use std::collections::HashMap;
    let inner = json.trim_start_matches('{').trim_end_matches('}');
    if inner.trim().is_empty() {
        return Ok(SerializedValue::Object(HashMap::new()));
    }
    let parts = split_json_values(inner);
    let mut map = HashMap::new();
    for part in parts {
        let part = part.trim();
        if let Some(colon) = find_colon(part) {
            let key_raw = part[..colon].trim();
            let val_raw = part[colon + 1..].trim();
            let key = key_raw.trim_matches('"').to_string();
            let val = parse_json(val_raw)?;
            map.insert(key, val);
        }
    }
    Ok(SerializedValue::Object(map))
}

fn split_json_values(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut start = 0;
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if in_string {
            if b == b'"' && (i == 0 || bytes[i - 1] != b'\\') {
                in_string = false;
            }
        } else {
            match b {
                b'"' => in_string = true,
                b'[' | b'{' => depth += 1,
                b']' | b'}' => depth -= 1,
                b',' if depth == 0 => {
                    parts.push(&s[start..i]);
                    start = i + 1;
                }
                _ => {}
            }
        }
    }
    if start < s.len() {
        parts.push(&s[start..]);
    }
    parts
}

fn find_colon(s: &str) -> Option<usize> {
    let mut in_string = false;
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if in_string {
            if b == b'"' && (i == 0 || bytes[i - 1] != b'\\') {
                in_string = false;
            }
        } else {
            match b {
                b'"' => in_string = true,
                b':' => return Some(i),
                _ => {}
            }
        }
    }
    None
}

impl std::fmt::Debug for Deserializer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Deserializer({})", self.data.to_json_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::value::SerializedValue;
    use std::collections::HashMap;

    #[test]
    fn test_deserialize_basic() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), SerializedValue::from("Alice"));
        map.insert("score".to_string(), SerializedValue::from(99i64));
        map.insert("alive".to_string(), SerializedValue::from(true));
        let d = Deserializer::new(SerializedValue::Object(map));
        assert_eq!(d.read_str("name"), Some("Alice"));
        assert_eq!(d.read_int("score"), Some(99));
        assert_eq!(d.read_bool("alive"), Some(true));
    }

    #[test]
    fn test_deserializer_has_key() {
        let mut map = HashMap::new();
        map.insert("x".to_string(), SerializedValue::from(1i64));
        let d = Deserializer::new(SerializedValue::Object(map));
        assert!(d.has_key("x"));
        assert!(!d.has_key("y"));
    }

    #[test]
    fn test_from_json_null() {
        let d = Deserializer::from_json("null").unwrap();
        assert!(d.get_value().is_null());
    }

    #[test]
    fn test_from_json_bool() {
        let d = Deserializer::from_json("true").unwrap();
        assert_eq!(d.get_value().as_bool(), Some(true));
    }

    #[test]
    fn test_from_json_int() {
        let d = Deserializer::from_json("42").unwrap();
        assert_eq!(d.get_value().as_int(), Some(42));
    }

    #[test]
    fn test_from_json_string() {
        let d = Deserializer::from_json("\"hello\"").unwrap();
        assert_eq!(d.get_value().as_str(), Some("hello"));
    }

    #[test]
    fn test_from_json_array() {
        let d = Deserializer::from_json("[1,2,3]").unwrap();
        let arr = d.get_value().as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_from_json_object() {
        let d = Deserializer::from_json("{\"x\":10,\"y\":20}").unwrap();
        assert_eq!(d.read_int("x"), Some(10));
        assert_eq!(d.read_int("y"), Some(20));
    }

    #[test]
    fn test_roundtrip() {
        let mut s = crate::serialization::serializer::Serializer::new();
        s.write_str("type", "player");
        s.write_int("hp", 100);
        s.write_float("x", 3.14);
        let json = s.to_json();
        let d = Deserializer::from_json(&json).unwrap();
        assert_eq!(d.read_str("type"), Some("player"));
        assert_eq!(d.read_int("hp"), Some(100));
        assert!((d.read_float("x").unwrap() - 3.14).abs() < 1e-9);
    }
}
