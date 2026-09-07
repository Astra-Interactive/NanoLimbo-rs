/// Where a chat message appears on the client.
///
/// From 1.19.1 the protocol keeps only the action bar distinction, so the other two
/// collapse into a single "system message" boolean.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatPosition {
    Chat,
    SystemMessage,
    ActionBar,
}

impl ChatPosition {
    /// The number the client reads before 1.19.1.
    pub const fn index(self) -> i32 {
        match self {
            Self::Chat => 0,
            Self::SystemMessage => 1,
            Self::ActionBar => 2,
        }
    }
}
