//! Packets sent while the connection is in the status state, answering a server ping.

mod status_player;
mod status_response;

pub use status_player::StatusPlayer;
pub use status_response::StatusResponse;
