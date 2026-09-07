/// What `start_app` reports back to the host.
///
/// A host that loads this library through a C ABI has no error type to inspect and no
/// standard error channel it can rely on, so every failure is flattened into a code it
/// can log. The numbers are part of the published interface: keep the existing ones
/// stable and append rather than renumber.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum StartStatus {
    /// The server ran and shut down when it was asked to.
    Ok = 0,
    /// The cancellation token pointer was null.
    NullToken = 1,
    /// The configuration directory was null, or was not valid UTF-8.
    InvalidConfigDirectory = 2,
    /// The configuration or the embedded resources could not be loaded.
    StartupFailed = 3,
    /// The async runtime could not be built.
    RuntimeUnavailable = 4,
    /// The configured address could not be listened on.
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

    /// The host logs these numbers and users quote them in bug reports, so a silent
    /// renumbering would make older reports mean something else.
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
