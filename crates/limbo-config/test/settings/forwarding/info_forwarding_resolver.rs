use crate::settings::forwarding::info_forwarding_resolver::token_lines;

#[test]
fn given_a_token_file_when_its_lines_are_read_then_comments_and_blanks_are_skipped() {
    let content = "# proxy one\nfirst\n\n   second   \n#trailing note\n";

    assert_eq!(token_lines(content), ["first", "second"]);
}

#[test]
fn given_a_file_of_only_comments_when_its_lines_are_read_then_there_are_no_tokens() {
    assert!(token_lines("# nothing here\n\n").is_empty());
}
