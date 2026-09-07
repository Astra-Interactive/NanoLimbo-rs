use std::str::FromStr;

use crate::settings_error::SettingsError;

/// How many notches the client draws across the boss bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossBarDivision {
    Solid,
    Dashes6,
    Dashes10,
    Dashes12,
    Dashes20,
}

impl BossBarDivision {
    /// The number the boss bar packet carries.
    pub const fn index(self) -> i32 {
        match self {
            Self::Solid => 0,
            Self::Dashes6 => 1,
            Self::Dashes10 => 2,
            Self::Dashes12 => 3,
            Self::Dashes20 => 4,
        }
    }
}

impl FromStr for BossBarDivision {
    type Err = SettingsError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_uppercase().as_str() {
            "SOLID" => Ok(Self::Solid),
            "DASHES_6" => Ok(Self::Dashes6),
            "DASHES_10" => Ok(Self::Dashes10),
            "DASHES_12" => Ok(Self::Dashes12),
            "DASHES_20" => Ok(Self::Dashes20),
            _ => Err(SettingsError::UnknownBossBarDivision {
                value: value.to_owned(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_a_dashed_name_when_parsed_then_the_division_is_recognised() {
        assert_eq!("dashes_10".parse().ok(), Some(BossBarDivision::Dashes10));
        assert_eq!("DASHES_20".parse().ok(), Some(BossBarDivision::Dashes20));
    }

    #[test]
    fn given_a_count_that_is_not_offered_when_parsed_then_it_is_rejected() {
        assert!("DASHES_7".parse::<BossBarDivision>().is_err());
    }
}
