use std::collections::{BTreeMap, HashMap};

pub type ValueVector = Vec<Value>;
pub type ValueMap = HashMap<String, Value>;
pub type ValueMapIntKey = BTreeMap<i32, Value>;

pub fn value_vector_null() -> ValueVector {
    Vec::new()
}

pub fn value_map_null() -> ValueMap {
    HashMap::new()
}

pub fn value_map_int_key_null() -> ValueMapIntKey {
    BTreeMap::new()
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Value {
    #[default]
    None,
    Byte(u8),
    Integer(i32),
    Unsigned(u32),
    Float(f32),
    Double(f64),
    Boolean(bool),
    String(String),
    Vector(ValueVector),
    Map(ValueMap),
    IntKeyMap(ValueMapIntKey),
}

impl Value {
    pub const VALUE_NULL: Value = Value::None;

    pub fn get_type(&self) -> ValueType {
        match self {
            Value::None => ValueType::None,
            Value::Byte(_) => ValueType::Byte,
            Value::Integer(_) => ValueType::Integer,
            Value::Unsigned(_) => ValueType::Unsigned,
            Value::Float(_) => ValueType::Float,
            Value::Double(_) => ValueType::Double,
            Value::Boolean(_) => ValueType::Boolean,
            Value::String(_) => ValueType::String,
            Value::Vector(_) => ValueType::Vector,
            Value::Map(_) => ValueType::Map,
            Value::IntKeyMap(_) => ValueType::IntKeyMap,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::None)
    }

    pub fn as_byte(&self) -> Option<u8> {
        match self {
            Value::Byte(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i32> {
        match self {
            Value::Integer(v) => Some(*v),
            Value::Byte(v) => Some(*v as i32),
            Value::Unsigned(v) => Some(*v as i32),
            Value::Boolean(v) => Some(if *v { 1 } else { 0 }),
            _ => None,
        }
    }

    pub fn as_unsigned(&self) -> Option<u32> {
        match self {
            Value::Unsigned(v) => Some(*v),
            Value::Byte(v) => Some(*v as u32),
            Value::Integer(v) => {
                if *v >= 0 {
                    Some(*v as u32)
                } else {
                    None
                }
            }
            Value::Boolean(v) => Some(if *v { 1 } else { 0 }),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f32> {
        match self {
            Value::Float(v) => Some(*v),
            Value::Double(v) => Some(*v as f32),
            Value::Integer(v) => Some(*v as f32),
            Value::Unsigned(v) => Some(*v as f32),
            _ => None,
        }
    }

    pub fn as_double(&self) -> Option<f64> {
        match self {
            Value::Double(v) => Some(*v),
            Value::Float(v) => Some(*v as f64),
            Value::Integer(v) => Some(*v as f64),
            Value::Unsigned(v) => Some(*v as f64),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(v) => Some(*v),
            Value::Integer(v) => Some(*v != 0),
            Value::Unsigned(v) => Some(*v != 0),
            Value::Float(v) => Some((*v != 0.0) && !v.is_nan()),
            Value::Double(v) => Some((*v != 0.0) && !v.is_nan()),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        match self {
            Value::String(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_value_vector(&self) -> Option<&ValueVector> {
        match self {
            Value::Vector(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_value_map(&self) -> Option<&ValueMap> {
        match self {
            Value::Map(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_int_key_map(&self) -> Option<&ValueMapIntKey> {
        match self {
            Value::IntKeyMap(v) => Some(v),
            _ => None,
        }
    }

    pub fn get_description(&self) -> String {
        match self {
            Value::None => "None".to_string(),
            Value::Byte(_) => "Byte".to_string(),
            Value::Integer(_) => "Integer".to_string(),
            Value::Unsigned(_) => "Unsigned".to_string(),
            Value::Float(_) => "Float".to_string(),
            Value::Double(_) => "Double".to_string(),
            Value::Boolean(_) => "Boolean".to_string(),
            Value::String(_) => "String".to_string(),
            Value::Vector(_) => "Vector".to_string(),
            Value::Map(_) => "Map".to_string(),
            Value::IntKeyMap(_) => "IntKeyMap".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    None,
    Byte,
    Integer,
    Unsigned,
    Float,
    Double,
    Boolean,
    String,
    Vector,
    Map,
    IntKeyMap,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_none() {
        let v = Value::None;
        assert!(v.is_null());
        assert_eq!(v.get_type(), ValueType::None);
    }

    #[test]
    fn test_value_byte() {
        let v = Value::Byte(42);
        assert!(!v.is_null());
        assert_eq!(v.get_type(), ValueType::Byte);
        assert_eq!(v.as_byte(), Some(42));
    }

    #[test]
    fn test_value_integer() {
        let v = Value::Integer(-123);
        assert_eq!(v.get_type(), ValueType::Integer);
        assert_eq!(v.as_int(), Some(-123));
    }

    #[test]
    fn test_value_unsigned() {
        let v = Value::Unsigned(456);
        assert_eq!(v.get_type(), ValueType::Unsigned);
        assert_eq!(v.as_unsigned(), Some(456));
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_value_float() {
        let v = Value::Float(3.14);
        assert_eq!(v.get_type(), ValueType::Float);
        assert_eq!(v.as_float(), Some(3.14));
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_value_double() {
        let v = Value::Double(2.718);
        assert_eq!(v.get_type(), ValueType::Double);
        assert_eq!(v.as_double(), Some(2.718));
    }

    #[test]
    fn test_value_boolean() {
        let v = Value::Boolean(true);
        assert_eq!(v.get_type(), ValueType::Boolean);
        assert_eq!(v.as_bool(), Some(true));
    }

    #[test]
    fn test_value_string() {
        let v = Value::String("hello".to_string());
        assert_eq!(v.get_type(), ValueType::String);
        assert_eq!(v.as_string(), Some(&"hello".to_string()));
    }

    #[test]
    fn test_value_vector() {
        let vec_val = vec![Value::Integer(1), Value::Integer(2)];
        let v = Value::Vector(vec_val.clone());
        assert_eq!(v.get_type(), ValueType::Vector);
        assert_eq!(v.as_value_vector(), Some(&vec_val));
    }

    #[test]
    fn test_value_map() {
        let mut map_val = HashMap::new();
        map_val.insert("key".to_string(), Value::Integer(123));
        let v = Value::Map(map_val);
        assert_eq!(v.get_type(), ValueType::Map);
        assert!(v.as_value_map().is_some());
    }

    #[test]
    fn test_value_int_key_map() {
        let mut map_val = BTreeMap::new();
        map_val.insert(1, Value::Integer(100));
        let v = Value::IntKeyMap(map_val);
        assert_eq!(v.get_type(), ValueType::IntKeyMap);
        assert!(v.as_int_key_map().is_some());
    }

    #[test]
    fn test_value_null_constant() {
        assert!(Value::VALUE_NULL.is_null());
    }
}
