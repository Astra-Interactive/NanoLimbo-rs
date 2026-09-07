//! A lightweight Minecraft limbo server.
//!
//! Exposed as a library so the runtime can be driven by tests: the login transcript suite
//! stands a real server up on an ephemeral port and walks every protocol version through
//! it. A binary alone could not be tested that way.

pub mod connection_task;
pub mod console;
pub mod forwarding_verifiers;
pub mod listener;
pub mod logging;
pub mod packet_snapshots;
pub mod prepared_server;
pub mod random_id_source;
pub mod server_context;
pub mod startup;
pub mod startup_error;
pub mod title_snapshots;
