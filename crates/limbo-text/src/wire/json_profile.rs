use limbo_protocol::version::ProtocolVersion;

use crate::chat::ClickEvent;

/// A set of serialization rules shared by a span of protocol versions.
///
/// The client's component parser changed shape four times. Each profile below is one of
/// those shapes, and the differences are wire-visible: emitting the wrong key name makes
/// the client ignore a hover event, and emitting a hex colour to a client that predates
/// them makes it ignore the colour.
///
/// Two of these profiles never put JSON on the wire for chat, since components travel as
/// NBT from 1.20.3. They still matter: the login disconnect and status response packets
/// serialize a component to a JSON string on every version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonProfile {
    /// Before 1.16. No hex colours, hover payload under `value`.
    Legacy,
    /// 1.16 through 1.20.2. Hex colours arrive, hover payload moves to `contents`.
    HexColours,
    /// 1.20.3 through 1.21.4. Unstyled text collapses to a bare string.
    CompactText,
    /// 1.21.5 and later. Event keys and payloads are renamed.
    RenamedEvents,
}

impl JsonProfile {
    pub fn for_version(version: ProtocolVersion) -> Self {
        if version >= ProtocolVersion::V1_21_5 {
            Self::RenamedEvents
        } else if version >= ProtocolVersion::V1_20_3 {
            Self::CompactText
        } else if version >= ProtocolVersion::V1_16 {
            Self::HexColours
        } else {
            Self::Legacy
        }
    }

    /// Whether an arbitrary 24-bit colour survives, or has to be reduced to one of the
    /// sixteen named colours the client has always understood.
    pub const fn emits_rgb(self) -> bool {
        !matches!(self, Self::Legacy)
    }

    /// Whether text with no style and no children collapses from `{"text":"x"}` to `"x"`.
    pub const fn compacts_plain_text(self) -> bool {
        matches!(self, Self::CompactText | Self::RenamedEvents)
    }

    pub const fn hover_event_key(self) -> &'static str {
        match self {
            Self::RenamedEvents => "hover_event",
            _ => "hoverEvent",
        }
    }

    /// The key the `show_text` payload sits under, which moved twice.
    pub const fn hover_text_key(self) -> &'static str {
        match self {
            Self::Legacy | Self::RenamedEvents => "value",
            Self::HexColours | Self::CompactText => "contents",
        }
    }

    pub const fn click_event_key(self) -> &'static str {
        match self {
            Self::RenamedEvents => "click_event",
            _ => "clickEvent",
        }
    }

    /// The key a click payload sits under. From 1.21.5 it is named after the action
    /// instead of being a generic `value`.
    pub const fn click_payload_key(self, event: &ClickEvent) -> &'static str {
        match self {
            Self::RenamedEvents => match event {
                ClickEvent::OpenUrl { .. } => "url",
                ClickEvent::RunCommand { .. } | ClickEvent::SuggestCommand { .. } => "command",
                ClickEvent::CopyToClipboard { .. } => "value",
            },
            _ => "value",
        }
    }
}
