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
