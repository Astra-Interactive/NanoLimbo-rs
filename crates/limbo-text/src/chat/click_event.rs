/// An action the client performs when the text is clicked.
///
/// MiniMessage can express more actions than this; the variants here are the ones a
/// limbo server has any use for, since it runs no commands and holds no books.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClickEvent {
    OpenUrl { url: String },
    RunCommand { command: String },
    SuggestCommand { command: String },
    CopyToClipboard { text: String },
}

impl ClickEvent {
    /// The `action` value written in JSON.
    pub const fn action(&self) -> &'static str {
        match self {
            Self::OpenUrl { .. } => "open_url",
            Self::RunCommand { .. } => "run_command",
            Self::SuggestCommand { .. } => "suggest_command",
            Self::CopyToClipboard { .. } => "copy_to_clipboard",
        }
    }

    /// The payload written alongside the action.
    pub fn value(&self) -> &str {
        match self {
            Self::OpenUrl { url } => url,
            Self::RunCommand { command } | Self::SuggestCommand { command } => command,
            Self::CopyToClipboard { text } => text,
        }
    }
}
