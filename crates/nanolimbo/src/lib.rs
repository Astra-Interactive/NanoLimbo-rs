//! A lightweight Minecraft limbo server.
//!
//! Exposed as a library so the runtime can be driven by tests: the login transcript suite
//! stands a real server up on an ephemeral port and walks every protocol version through
//! it. A binary alone could not be tested that way.

pub mod connection {
    //! Serving clients: the accept loop, and the task that carries one connection from its
    //! handshake to its disconnect.

    mod connection_task;
    mod ending;
    mod forwarded_from_proxy;
    mod listener;
    mod session;

    pub(crate) use connection_task::serve;
    pub(crate) use ending::Ending;
    pub(crate) use forwarded_from_proxy::ForwardedFromProxy;
    pub use listener::accept_until_shutdown;
    pub(crate) use session::{Connection, Session};
}

pub mod di {
    //! What every connection shares, built once at startup and handed down.

    mod forwarding_verifiers;
    mod server_context;

    pub use forwarding_verifiers::ForwardingVerifiers;
    pub use server_context::ServerContext;
}

pub mod id {
    //! The random source of the ids the protocol asks for.

    mod random_id_source;

    pub use random_id_source::RandomIdSource;
}

pub mod lifecycle {
    //! Starting the server, running it until it is told to stop, and stopping it.

    mod console;
    mod logging;
    mod prepared_server;
    mod shutdown_source;
    mod startup;
    mod startup_error;

    pub(crate) use console::{read_console_lines, run_console};
    pub use logging::{install_logging, level_from_debug_setting};
    pub use prepared_server::PreparedServer;
    pub use shutdown_source::ShutdownSource;
    pub use startup::{VERSION, prepare, serve};
    pub use startup_error::StartupError;
}

pub mod snapshot {
    //! Packets encoded once at startup for every protocol version, because every player
    //! is sent the same ones.

    mod packet_snapshots;
    mod title_snapshots;

    pub use packet_snapshots::PacketSnapshots;
    pub use title_snapshots::TitleSnapshots;
}

#[cfg(test)]
#[path = "../test/lib.rs"]
mod test;
