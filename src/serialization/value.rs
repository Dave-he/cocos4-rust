use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum SerializedValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<SerializedValue>),
    Object(HashMap<String, SerializedValue>),
}

impl SerializedValue {
    pub fn is_null(&self) -> bool {
        matches!(self, SerializedValue::Null)
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let SerializedValue::Bool(v) = self { Some(*v) } else { None }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            SerializedValue::Int(v) => Some(*v),
            SerializedValue::Float(v) => Some(*v as i64),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            SerializedValue::Float(v) => Some(*v),
            SerializedValue::Int(v) => Some(*v as f64),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let SerializedValue::String(s) = self { Some(s.as_str()) } else { None }
    }

    pub fn as_array(&self) -> Option<&Vec<SerializedValue>> {
        if let SerializedValue::Array(a) = self { Some(a) } else { None }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, SerializedValue>> {
        if let SerializedValue::Object(o) = self { Some(o) } else { None }
    }

    pub fn get(&self, key: &str) -> Option<&SerializedValue> {
        if let SerializedValue::Object(o) = self {
            o.get(key)
        } else {
            None
        }
    }

    pub fn get_index(&self, idx: usize) -> Option<&SerializedValue> {
        if let SerializedValue::Array(a) = self {
            a.get(idx)
        } else {
            None
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            SerializedValue::Null => "null",
            SerializedValue::Bool(_) => "bool",
            SerializedValue::Int(_) => "int",
            SerializedValue::Float(_) => "float",
            SerializedValue::String(_) => "string",
            SerializedValue::Array(_) => "array",
            SerializedValue::Object(_) => "object",
        }
    }

    pub fn to_json_string(&self) -> String {
        match self {
            SerializedValue::Null => "null".to_string(),
            SerializedValue::Bool(v) => v.to_string(),
            SerializedValue::Int(v) => v.to_string(),
            SerializedValue::Float(v) => {
                if v.fract() == 0.0 && v.is_finite() {
                    format!("{:.1}", v)
                } else {
                    v.to_string()
                }
            }
            SerializedValue::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            SerializedValue::Array(a) => {
                let items: Vec<String> = a.iter().map(|v| v.to_json_string()).collect();
                format!("[{}]", items.join(","))
            }
            SerializedValue::Object(o) => {
                let mut pairs: Vec<String> = o.iter()
                    .map(|(k, v)| format!("\"{}\":{}", k, v.to_json_string()))
                    .collect();
                pairs.sort();
                format!("{{{}}}", pairs.join(","))
            }
        }
    }
}

impl From<bool> for SerializedValue {
    fn from(v: bool) -> Self { SerializedValue::Bool(v) }
}

impl From<i32> for SerializedValue {
    fn from(v: i32) -> Self { SerializedValue::Int(v as i64) }
}

impl From<i64> for SerializedValue {
    fn from(v: i64) -> Self { SerializedValue::Int(v) }
}

impl From<f32> for SerializedValue {
    fn from(v: f32) -> Self { SerializedValue::Float(v as f64) }
}

impl From<f64> for SerializedValue {
    fn from(v: f64) -> Self { SerializedValue::Float(v) }
}

impl From<&str> for SerializedValue {
    fn from(v: &str) -> Self { SerializedValue::String(v.to_string()) }
}

impl From<String> for SerializedValue {
    fn from(v: String) -> Self { SerializedValue::String(v) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null() {
        let v = SerializedValue::Null;
        assert!(v.is_null());
        assert_eq!(v.to_json_string(), "null");
    }

    #[test]
    fn test_bool() {
        let v = SerializedValue::from(true);
        assert_eq!(v.as_bool(), Some(true));
        assert_eq!(v.to_json_string(), "true");
    }

    #[test]
    fn test_int() {
        let v = SerializedValue::from(42i64);
        assert_eq!(v.as_int(), Some(42));
        assert_eq!(v.to_json_string(), "42");
    }

    #[test]
    fn test_float() {
        let v = SerializedValue::from(3.14f64);
        assert!((v.as_f64().unwrap() - 3.14).abs() < 1e-9);
    }

    #[test]
    fn test_string() {
        let v = SerializedValue::from("hello");
        assert_eq!(v.as_str(), Some("hello"));
        assert_eq!(v.to_json_string(), "\"hello\"");
    }

    #[test]
    fn test_array() {
        let v = SerializedValue::Array(vec![
            SerializedValue::from(1i64),
            SerializedValue::from(2i64),
        ]);
        assert_eq!(v.get_index(0).unwrap().as_int(), Some(1));
        assert_eq!(v.to_json_string(), "[1,2]");
    }

    #[test]
    fn test_object() {
        let mut map = HashMap::new();
        map.insert("x".to_string(), SerializedValue::from(10i64));
        map.insert("y".to_string(), SerializedValue::from(20i64));
        let v = SerializedValue::Object(map);
        assert_eq!(v.get("x").unwrap().as_int(), Some(10));
    }

    #[test]
    fn test_type_name() {
        assert_eq!(SerializedValue::Null.type_name(), "null");
        assert_eq!(SerializedValue::from(true).type_name(), "bool");
        assert_eq!(SerializedValue::from(1i64).type_name(), "int");
        assert_eq!(SerializedValue::from(1.0f64).type_name(), "float");
        assert_eq!(SerializedValue::from("s").type_name(), "string");
    }
}
