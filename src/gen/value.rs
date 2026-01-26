use serde::de::{self, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::forward_to_deserialize_any;
use std::collections::HashMap;
use std::fmt;

/// A JSON-like value that can hold NaN, Infinity, and large integers.
#[derive(Clone, Debug)]
pub enum HegelValue {
    Null,
    Bool(bool),
    Number(f64),
    /// Large integer that doesn't fit in f64 precisely (abs >= 2^63)
    BigInt(String),
    String(String),
    Array(Vec<HegelValue>),
    Object(HashMap<String, HegelValue>),
}

impl From<serde_json::Value> for HegelValue {
    fn from(v: serde_json::Value) -> Self {
        match v {
            serde_json::Value::Null => HegelValue::Null,
            serde_json::Value::Bool(b) => HegelValue::Bool(b),
            serde_json::Value::Number(n) => HegelValue::Number(n.as_f64().unwrap_or(0.0)),
            serde_json::Value::String(s) => HegelValue::String(s),
            serde_json::Value::Array(arr) => {
                HegelValue::Array(arr.into_iter().map(HegelValue::from).collect())
            }
            serde_json::Value::Object(map) => {
                // Check for special object wrappers
                if map.len() == 1 {
                    if let Some(serde_json::Value::String(s)) = map.get("$float") {
                        return match s.as_str() {
                            "inf" => HegelValue::Number(f64::INFINITY),
                            "-inf" => HegelValue::Number(f64::NEG_INFINITY),
                            "nan" => HegelValue::Number(f64::NAN),
                            _ => HegelValue::Number(f64::NAN), // fallback
                        };
                    } else if let Some(serde_json::Value::String(s)) = map.get("$integer") {
                        return HegelValue::BigInt(s.clone());
                    }
                }
                HegelValue::Object(
                    map.into_iter()
                        .map(|(k, v)| (k, HegelValue::from(v)))
                        .collect(),
                )
            }
        }
    }
}

#[derive(Debug)]
pub struct HegelValueError(String);

impl fmt::Display for HegelValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for HegelValueError {}

impl de::Error for HegelValueError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        HegelValueError(msg.to_string())
    }
}

impl<'de> Deserializer<'de> for HegelValue {
    type Error = HegelValueError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self {
            HegelValue::Null => visitor.visit_unit(),
            HegelValue::Bool(b) => visitor.visit_bool(b),
            HegelValue::Number(n) => {
                // For whole numbers that fit in i64, use visit_i64 so integer
                // deserialization works. NaN/Inf have fract() != 0, so they
                // go to visit_f64.
                if n.fract() == 0.0 && n >= i64::MIN as f64 && n <= i64::MAX as f64 {
                    visitor.visit_i64(n as i64)
                } else {
                    visitor.visit_f64(n)
                }
            }
            HegelValue::BigInt(s) => {
                // Parse the string and use the smallest visitor type that fits.
                // This ensures compatibility with serde's primitive deserializers.
                if let Ok(n) = s.parse::<u64>() {
                    visitor.visit_u64(n)
                } else if let Ok(n) = s.parse::<i64>() {
                    visitor.visit_i64(n)
                } else if let Ok(n) = s.parse::<u128>() {
                    visitor.visit_u128(n)
                } else if let Ok(n) = s.parse::<i128>() {
                    visitor.visit_i128(n)
                } else {
                    Err(HegelValueError(format!("invalid big integer value: {}", s)))
                }
            }
            HegelValue::String(s) => visitor.visit_string(s),
            HegelValue::Array(arr) => visitor.visit_seq(HegelSeqAccess {
                iter: arr.into_iter(),
            }),
            HegelValue::Object(map) => visitor.visit_map(HegelMapAccess {
                iter: map.into_iter(),
                value: None,
            }),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self {
            HegelValue::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct HegelSeqAccess {
    iter: std::vec::IntoIter<HegelValue>,
}

impl<'de> SeqAccess<'de> for HegelSeqAccess {
    type Error = HegelValueError;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }
}

struct HegelMapAccess {
    iter: std::collections::hash_map::IntoIter<String, HegelValue>,
    value: Option<HegelValue>,
}

impl<'de> MapAccess<'de> for HegelMapAccess {
    type Error = HegelValueError;

    fn next_key_seed<K: de::DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(StringDeserializer(key)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V: de::DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        let value = self
            .value
            .take()
            .expect("next_value called before next_key");
        seed.deserialize(value)
    }
}

struct StringDeserializer(String);

impl<'de> Deserializer<'de> for StringDeserializer {
    type Error = HegelValueError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_string(self.0)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

pub fn from_hegel_value<T: de::DeserializeOwned>(value: HegelValue) -> Result<T, HegelValueError> {
    T::deserialize(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_f64() {
        let v = HegelValue::Number(42.5);
        let result: f64 = from_hegel_value(v).unwrap();
        assert_eq!(result, 42.5);
    }

    #[test]
    fn test_deserialize_nan() {
        let v = HegelValue::Number(f64::NAN);
        let result: f64 = from_hegel_value(v).unwrap();
        assert!(result.is_nan());
    }

    #[test]
    fn test_deserialize_infinity() {
        let v = HegelValue::Number(f64::INFINITY);
        let result: f64 = from_hegel_value(v).unwrap();
        assert!(result.is_infinite() && result.is_sign_positive());
    }

    #[test]
    fn test_deserialize_neg_infinity() {
        let v = HegelValue::Number(f64::NEG_INFINITY);
        let result: f64 = from_hegel_value(v).unwrap();
        assert!(result.is_infinite() && result.is_sign_negative());
    }

    #[test]
    fn test_deserialize_vec_f64() {
        let v = HegelValue::Array(vec![
            HegelValue::Number(1.0),
            HegelValue::Number(f64::NAN),
            HegelValue::Number(f64::INFINITY),
        ]);
        let result: Vec<f64> = from_hegel_value(v).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], 1.0);
        assert!(result[1].is_nan());
        assert!(result[2].is_infinite());
    }

    #[test]
    fn test_from_serde_json_object_wrappers() {
        let json =
            serde_json::json!([1.0, {"$float": "nan"}, {"$float": "inf"}, {"$float": "-inf"}]);
        let hegel = HegelValue::from(json);
        let result: Vec<f64> = from_hegel_value(hegel).unwrap();
        assert_eq!(result[0], 1.0);
        assert!(result[1].is_nan());
        assert!(result[2].is_infinite() && result[2].is_sign_positive());
        assert!(result[3].is_infinite() && result[3].is_sign_negative());
    }

    #[test]
    fn test_from_serde_json_big_integer() {
        // Value larger than 2^63
        let json = serde_json::json!({"$integer": "9223372036854776833"});
        let hegel = HegelValue::from(json);
        let result: u64 = from_hegel_value(hegel).unwrap();
        assert_eq!(result, 9223372036854776833u64);
    }

    #[test]
    fn test_from_serde_json_big_negative_integer() {
        // Large negative value
        let json = serde_json::json!({"$integer": "-9223372036854776833"});
        let hegel = HegelValue::from(json);
        let result: i128 = from_hegel_value(hegel).unwrap();
        assert_eq!(result, -9223372036854776833i128);
    }

    #[test]
    fn test_deserialize_struct() {
        #[derive(serde::Deserialize, Debug)]
        struct TestStruct {
            value: f64,
            name: String,
        }

        let v = HegelValue::Object(HashMap::from([
            ("value".to_string(), HegelValue::Number(f64::NAN)),
            ("name".to_string(), HegelValue::String("test".to_string())),
        ]));
        let result: TestStruct = from_hegel_value(v).unwrap();
        assert!(result.value.is_nan());
        assert_eq!(result.name, "test");
    }
}
