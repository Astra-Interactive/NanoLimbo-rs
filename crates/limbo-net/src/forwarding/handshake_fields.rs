/// What a proxy separates the fields it appends to the handshake host with.
const FIELD_SEPARATOR: char = '\0';

/// Splits a handshake host into the fields a proxy packed into it.
///
/// Trailing empty fields are dropped, because Java's `String.split` drops them and the
/// Java implementation's field counts are written against that behaviour: a host ending
/// in a separator has to count as though it did not.
pub(crate) fn split_forwarded_fields(host: &str) -> Vec<&str> {
    let mut fields: Vec<&str> = host.split(FIELD_SEPARATOR).collect();

    while fields.last().is_some_and(|field| field.is_empty()) {
        fields.pop();
    }

    fields
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
