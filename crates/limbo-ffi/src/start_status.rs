/// Published interface: keep the existing numbers and append, never renumber.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum StartStatus {
    Ok = 0,
    NullToken = 1,
    /// Null, or not valid UTF-8.
    InvalidConfigDirectory = 2,
    StartupFailed = 3,
    RuntimeUnavailable = 4,
    BindFailed = 5,
}

impl StartStatus {
    pub const fn code(self) -> i32 {
        self as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_the_published_statuses_when_read_as_codes_then_they_keep_their_numbers() {
        assert_eq!(StartStatus::Ok.code(), 0);
        assert_eq!(StartStatus::NullToken.code(), 1);
        assert_eq!(StartStatus::InvalidConfigDirectory.code(), 2);
        assert_eq!(StartStatus::StartupFailed.code(), 3);
        assert_eq!(StartStatus::RuntimeUnavailable.code(), 4);
        assert_eq!(StartStatus::BindFailed.code(), 5);
    }
}
