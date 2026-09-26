/// A command typed into the server console.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleCommand {
    Help,
    Connections,
    Memory,
    Version,
    Stop,
}

impl ConsoleCommand {
    pub const ALL: [Self; 5] = [
        Self::Help,
        Self::Connections,
        Self::Memory,
        Self::Version,
        Self::Stop,
    ];

    /// The words that invoke this command. The first is the one `help` advertises.
    pub const fn names(self) -> &'static [&'static str] {
        match self {
            Self::Help => &["help"],
            Self::Connections => &["conn"],
            Self::Memory => &["mem"],
            Self::Version => &["version", "ver"],
            Self::Stop => &["stop"],
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Help => "Show this message",
            Self::Connections => "Display connections count",
            Self::Memory => "Display memory usage",
            Self::Version => "Display limbo version",
            Self::Stop => "Stop the server",
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        let typed = input.trim().to_ascii_lowercase();
        Self::ALL
            .into_iter()
            .find(|command| command.names().contains(&typed.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
