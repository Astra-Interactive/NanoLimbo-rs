use crate::settings_error::SettingsError;

/// How full the boss bar is drawn, as a fraction of its width.
///
/// The client draws anything outside `0.0..=1.0` as a bar that overflows or vanishes, so
/// the range is checked once here and the type carries the guarantee afterwards.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct BossBarHealth(f32);

impl BossBarHealth {
    pub fn new(value: f32) -> Result<Self, SettingsError> {
        if !(0.0..=1.0).contains(&value) {
            return Err(SettingsError::BossBarHealthOutOfRange { health: value });
        }
        Ok(Self(value))
    }

    pub const fn value(self) -> f32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_the_ends_of_the_range_when_validated_then_both_are_accepted() {
        assert_eq!(
            BossBarHealth::new(0.0).map(BossBarHealth::value).ok(),
            Some(0.0)
        );
        assert_eq!(
            BossBarHealth::new(1.0).map(BossBarHealth::value).ok(),
            Some(1.0)
        );
    }

    #[test]
    fn given_a_value_just_outside_the_range_when_validated_then_it_is_rejected() {
        assert!(BossBarHealth::new(-0.1).is_err());
        assert!(BossBarHealth::new(1.1).is_err());
    }

    /// Java compares with `<` and `>`, both of which are false for a NaN, so a boss bar
    /// configured with `.nan` reached the client and drew nothing.
    #[test]
    fn given_a_nan_when_validated_then_it_is_rejected() {
        assert!(BossBarHealth::new(f32::NAN).is_err());
    }
}
