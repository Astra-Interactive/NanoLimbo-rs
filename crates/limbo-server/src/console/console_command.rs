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
