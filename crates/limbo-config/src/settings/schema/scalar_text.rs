use serde::Deserialize;
use serde::de::{Deserializer, Error as DeserializeError};
use serde_yaml_ng::Value;

/// A text setting, accepted whatever YAML decided its scalar type was.
///
/// `version: 1.20` is a float to YAML and a string to everyone else. The reference
/// implementation coerced any scalar to text, and a configuration that has been working
/// for years must keep working, so the same coercion happens here.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScalarText(String);

impl ScalarText {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for ScalarText {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Value::deserialize(deserializer)? {
            Value::Null => Ok(Self(String::new())),
            Value::Bool(flag) => Ok(Self(flag.to_string())),
            Value::Number(number) => Ok(Self(number.to_string())),
            Value::String(text) => Ok(Self(text)),
            _ => Err(D::Error::custom("expected a single value")),
        }
    }
}
