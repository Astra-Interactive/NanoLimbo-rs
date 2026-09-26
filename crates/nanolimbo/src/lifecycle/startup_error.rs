use std::net::SocketAddr;

/// Anything that stops the server before it has served anything.
#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    #[error("configuration: {0}")]
    Config(#[from] limbo_config::ConfigError),

    #[error("embedded dimension data: {0}")]
    Resources(#[from] limbo_world::ResourceLoadError),

    #[error("dimension {0}")]
    Dimension(#[from] limbo_world::DimensionLookupError),

    #[error("packet encoding: {0}")]
    Encoding(#[from] limbo_packet::PacketEncodeError),

    #[error("could not build the async runtime: {0}")]
    Runtime(std::io::Error),

    #[error("could not listen on {address}: {source}")]
    Bind {
        address: SocketAddr,
        source: std::io::Error,
    },
}
