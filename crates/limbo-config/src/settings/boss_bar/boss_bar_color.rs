use std::str::FromStr;

use crate::settings::SettingsError;

/// The colour the client fills the boss bar with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossBarColor {
    Pink,
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    White,
}

impl BossBarColor {
    /// The number the boss bar packet carries. Defined by the protocol, so it is part of
    /// the domain rather than of the encoder.
    pub const fn index(self) -> i32 {
        match self {
            Self::Pink => 0,
            Self::Blue => 1,
            Self::Red => 2,
            Self::Green => 3,
            Self::Yellow => 4,
            Self::Purple => 5,
            Self::White => 6,
        }
    }
}

impl FromStr for BossBarColor {
    type Err = SettingsError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_uppercase().as_str() {
            "PINK" => Ok(Self::Pink),
            "BLUE" => Ok(Self::Blue),
            "RED" => Ok(Self::Red),
            "GREEN" => Ok(Self::Green),
            "YELLOW" => Ok(Self::Yellow),
            "PURPLE" => Ok(Self::Purple),
            "WHITE" => Ok(Self::White),
            _ => Err(SettingsError::UnknownBossBarColor {
                value: value.to_owned(),
            }),
        }
    }
}
