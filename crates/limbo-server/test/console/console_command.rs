use crate::console::ConsoleCommand;

#[test]
fn given_a_command_with_surrounding_space_or_case_when_parsed_then_it_still_resolves() {
    assert_eq!(ConsoleCommand::parse("  STOP "), Some(ConsoleCommand::Stop));
}

#[test]
fn given_an_alias_when_parsed_then_it_resolves_to_the_same_command() {
    assert_eq!(ConsoleCommand::parse("ver"), Some(ConsoleCommand::Version));
    assert_eq!(
        ConsoleCommand::parse("version"),
        Some(ConsoleCommand::Version)
    );
}

#[test]
fn given_something_that_is_not_a_command_when_parsed_then_nothing_resolves() {
    assert_eq!(ConsoleCommand::parse(""), None);
    assert_eq!(ConsoleCommand::parse("halt"), None);
}

#[test]
fn given_the_command_table_when_scanned_then_no_two_commands_share_a_name() {
    let mut seen = Vec::new();
    for command in ConsoleCommand::ALL {
        for name in command.names() {
            assert!(!seen.contains(name), "{name} is claimed twice");
            seen.push(name);
        }
    }
}
