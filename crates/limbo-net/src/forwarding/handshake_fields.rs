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
