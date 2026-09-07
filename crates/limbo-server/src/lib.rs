//! The connection lifecycle: what a client sends, and what the server does about it.
//!
//! Decoding and decision-making live here as pure logic with no sockets in sight, so the
//! whole state machine is exercised in host tests. The runtime that owns the sockets
//! consumes it.

pub mod client_intent;
pub mod connected_player;
pub mod connection_action;
pub mod connection_flow;
pub mod connection_id;
pub mod connection_registry;
pub mod console_command;
pub mod forwarding_mode;
pub mod game_profile;
pub mod handshake;
pub mod login_plugin_response;
pub mod login_start;
pub mod memory_usage;
pub mod plugin_message;
pub mod server_policy;
pub mod serverbound_packet;
