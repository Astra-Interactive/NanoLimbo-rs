/// Which way a packet travels, from the server's point of view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketDirection {
    /// Sent by the client, decoded by the server.
    ServerBound,
    /// Built by the server, sent to the client.
    ClientBound,
}

impl PacketDirection {
    pub const ALL: [Self; 2] = [Self::ServerBound, Self::ClientBound];
}
