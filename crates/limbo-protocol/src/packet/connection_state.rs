/// Phase of the connection, which decides how packet ids are interpreted.
///
/// A connection walks Handshaking into either Status or Login; from 1.20.2 Login leads
/// through Configuration before Play, and older clients go straight to Play.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionState {
    Handshaking,
    Status,
    Login,
    Configuration,
    Play,
}

impl ConnectionState {
    pub const ALL: [Self; 5] = [
        Self::Handshaking,
        Self::Status,
        Self::Login,
        Self::Configuration,
        Self::Play,
    ];
}
