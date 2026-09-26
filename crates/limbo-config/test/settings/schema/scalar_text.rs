use crate::settings::schema::ScalarText;

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
