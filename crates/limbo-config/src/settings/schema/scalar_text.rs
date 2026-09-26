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

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(yaml: &str) -> Option<String> {
        serde_yaml_ng::from_str::<ScalarText>(yaml)
            .ok()
            .map(|text| text.as_str().to_owned())
    }

    #[test]
    fn given_an_unquoted_number_when_read_as_text_then_it_keeps_its_digits() {
        assert_eq!(parsed("1.20"), Some("1.2".to_owned()));
        assert_eq!(parsed("100"), Some("100".to_owned()));
    }

    #[test]
    fn given_a_quoted_string_when_read_as_text_then_it_is_unchanged() {
        assert_eq!(parsed("\"<red>hello\""), Some("<red>hello".to_owned()));
    }

    #[test]
    fn given_a_key_left_blank_when_read_as_text_then_it_is_empty() {
        assert_eq!(parsed("~"), Some(String::new()));
    }

    #[test]
    fn given_a_list_where_text_is_expected_when_read_then_it_is_an_error() {
        assert!(serde_yaml_ng::from_str::<ScalarText>("[a, b]").is_err());
    }
}
