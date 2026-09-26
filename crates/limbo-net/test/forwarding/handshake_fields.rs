use crate::forwarding::handshake_fields::split_forwarded_fields;

#[test]
fn given_a_plain_host_when_split_then_it_is_the_only_field() {
    assert_eq!(split_forwarded_fields("example.com"), vec!["example.com"]);
}

#[test]
fn given_forwarded_fields_when_split_then_each_one_is_returned() {
    assert_eq!(
        split_forwarded_fields("example.com\u{0}127.0.0.1\u{0}uuid"),
        vec!["example.com", "127.0.0.1", "uuid"]
    );
}

#[test]
fn given_a_trailing_separator_when_split_then_the_empty_field_is_dropped() {
    assert_eq!(
        split_forwarded_fields("example.com\u{0}127.0.0.1\u{0}uuid\u{0}"),
        vec!["example.com", "127.0.0.1", "uuid"]
    );
}

#[test]
fn given_an_empty_field_in_the_middle_when_split_then_it_is_kept() {
    assert_eq!(
        split_forwarded_fields("example.com\u{0}\u{0}uuid"),
        vec!["example.com", "", "uuid"]
    );
}

#[test]
fn given_an_empty_host_when_split_then_there_are_no_fields() {
    assert!(split_forwarded_fields("").is_empty());
}
