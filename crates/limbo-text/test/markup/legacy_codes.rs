use crate::markup::legacy_codes_to_tags;

#[test]
fn given_legacy_colour_and_format_codes_when_translated_then_they_become_tags() {
    assert_eq!(
        legacy_codes_to_tags("&aGreen &lBold&r plain"),
        "<green>Green <bold>Bold<reset> plain"
    );
}

#[test]
fn given_the_bukkit_hex_form_when_translated_then_it_becomes_one_hex_tag() {
    assert_eq!(
        legacy_codes_to_tags("&x&f&f&0&0&0&0hex legacy"),
        "<#ff0000>hex legacy"
    );
}

#[test]
fn given_a_section_sign_when_translated_then_it_is_treated_like_an_ampersand() {
    assert_eq!(legacy_codes_to_tags("\u{00A7}aGreen"), "<green>Green");
}

#[test]
fn given_a_marker_that_starts_nothing_when_translated_then_it_stays_literal() {
    assert_eq!(legacy_codes_to_tags("100% & rising"), "100% & rising");
    assert_eq!(legacy_codes_to_tags("trailing &"), "trailing &");
}
