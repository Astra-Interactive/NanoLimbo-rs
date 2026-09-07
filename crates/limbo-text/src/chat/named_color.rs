use crate::chat::rgb_color::RgbColor;

/// The sixteen colours the protocol has carried by name since the beginning.
///
/// Clients older than 1.16 understand only these, so an arbitrary [`RgbColor`] has to be
/// reduced to the nearest one before being sent to them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NamedColor {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
}

impl NamedColor {
    pub const ALL: [Self; 16] = [
        Self::Black,
        Self::DarkBlue,
        Self::DarkGreen,
        Self::DarkAqua,
        Self::DarkRed,
        Self::DarkPurple,
        Self::Gold,
        Self::Gray,
        Self::DarkGray,
        Self::Blue,
        Self::Green,
        Self::Aqua,
        Self::Red,
        Self::LightPurple,
        Self::Yellow,
        Self::White,
    ];

    /// The name used in JSON and in MiniMessage tags.
    pub const fn key(self) -> &'static str {
        match self {
            Self::Black => "black",
            Self::DarkBlue => "dark_blue",
            Self::DarkGreen => "dark_green",
            Self::DarkAqua => "dark_aqua",
            Self::DarkRed => "dark_red",
            Self::DarkPurple => "dark_purple",
            Self::Gold => "gold",
            Self::Gray => "gray",
            Self::DarkGray => "dark_gray",
            Self::Blue => "blue",
            Self::Green => "green",
            Self::Aqua => "aqua",
            Self::Red => "red",
            Self::LightPurple => "light_purple",
            Self::Yellow => "yellow",
            Self::White => "white",
        }
    }

    /// The legacy formatting code, as in `&a` or `§a`.
    pub const fn legacy_code(self) -> char {
        match self {
            Self::Black => '0',
            Self::DarkBlue => '1',
            Self::DarkGreen => '2',
            Self::DarkAqua => '3',
            Self::DarkRed => '4',
            Self::DarkPurple => '5',
            Self::Gold => '6',
            Self::Gray => '7',
            Self::DarkGray => '8',
            Self::Blue => '9',
            Self::Green => 'a',
            Self::Aqua => 'b',
            Self::Red => 'c',
            Self::LightPurple => 'd',
            Self::Yellow => 'e',
            Self::White => 'f',
        }
    }

    pub const fn rgb(self) -> RgbColor {
        match self {
            Self::Black => RgbColor::from_packed(0x000000),
            Self::DarkBlue => RgbColor::from_packed(0x0000AA),
            Self::DarkGreen => RgbColor::from_packed(0x00AA00),
            Self::DarkAqua => RgbColor::from_packed(0x00AAAA),
            Self::DarkRed => RgbColor::from_packed(0xAA0000),
            Self::DarkPurple => RgbColor::from_packed(0xAA00AA),
            Self::Gold => RgbColor::from_packed(0xFFAA00),
            Self::Gray => RgbColor::from_packed(0xAAAAAA),
            Self::DarkGray => RgbColor::from_packed(0x555555),
            Self::Blue => RgbColor::from_packed(0x5555FF),
            Self::Green => RgbColor::from_packed(0x55FF55),
            Self::Aqua => RgbColor::from_packed(0x55FFFF),
            Self::Red => RgbColor::from_packed(0xFF5555),
            Self::LightPurple => RgbColor::from_packed(0xFF55FF),
            Self::Yellow => RgbColor::from_packed(0xFFFF55),
            Self::White => RgbColor::from_packed(0xFFFFFF),
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|color| color.key() == key)
    }

    pub fn from_legacy_code(code: char) -> Option<Self> {
        let lowercase = code.to_ascii_lowercase();
        Self::ALL
            .into_iter()
            .find(|color| color.legacy_code() == lowercase)
    }
}
