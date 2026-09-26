//! The connection lifecycle: what a client sends, and what the server does about it.
//!
//! Decoding and decision-making live here as pure logic with no sockets in sight, so the
//! whole state machine is exercised in host tests. The runtime that owns the sockets
//! consumes it.

pub mod connection {
    //! The per-connection state machine: what the server does with each packet a client
    //! sends, under the policy the configuration sets.

    mod connection_action;
    pub(crate) mod connection_flow;
    mod forwarding_mode;
    mod server_policy;

    pub use connection_action::ConnectionAction;
    pub use connection_flow::ConnectionFlow;
    pub use forwarding_mode::ForwardingMode;
    pub use server_policy::ServerPolicy;
}

pub mod console {
    //! The operator's console: the commands it understands and the figures it reports.

    mod console_command;
    mod memory_usage;

    pub use console_command::ConsoleCommand;
    pub use memory_usage::MemoryUsage;
}

pub mod id {
    //! The values the server would otherwise draw from a random number generator.

    mod fixed_id_source;
    mod id_source;

    pub use fixed_id_source::FixedIdSource;
    pub use id_source::IdSource;
}

pub mod player {
    //! Who a player is, and which players are online right now.

    mod connected_player;
    mod connection_id;
    mod connection_registry;
    mod game_profile;

    pub use connected_player::ConnectedPlayer;
    pub use connection_id::ConnectionId;
    pub use connection_registry::ConnectionRegistry;
    pub use game_profile::GameProfile;
}

pub mod serverbound {
    //! Packets a client sends, decoded from untrusted bytes into the few the server acts on.

    mod client_intent;
    mod handshake;
    mod login_plugin_response;
    mod login_start;
    mod plugin_message;
    mod serverbound_packet;

    pub use client_intent::ClientIntent;
    pub use handshake::Handshake;
    pub use login_plugin_response::LoginPluginResponse;
    pub use login_start::LoginStart;
    pub use plugin_message::PluginMessage;
    pub use serverbound_packet::ServerBoundPacket;
}

#[cfg(test)]
#[path = "../test/lib.rs"]
mod test;
