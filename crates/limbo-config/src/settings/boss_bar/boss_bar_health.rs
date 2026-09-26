use crate::settings::SettingsError;

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
