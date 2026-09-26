use std::net::SocketAddr;

use limbo_protocol::packet::{ConnectionState, PacketDirection, PacketKind, PacketRoute};
use limbo_protocol::version::ProtocolVersion;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Nothing a limbo server sends approaches this, so a longer frame means the connection
/// has lost sync and there is no point reading further.
const MAX_FRAME_LENGTH: usize = 8 * 1024 * 1024;

fn write_var_int(target: &mut Vec<u8>, value: i32) {
    let mut remaining = value as u32;
    loop {
        if remaining & !0x7F == 0 {
            target.push(remaining as u8);
            return;
        }
        target.push((remaining as u8 & 0x7F) | 0x80);
        remaining >>= 7;
    }
}

fn write_string(target: &mut Vec<u8>, text: &str) {
    write_var_int(target, text.len() as i32);
    target.extend_from_slice(text.as_bytes());
}

/// A client that logs in and then stays there, which is what a limbo server holds.
///
/// Packet ids come from the same tables the server uses, so this keeps working when a new
/// Minecraft version shifts them — a benchmark with its own hardcoded ids would quietly
/// start measuring a failed login instead.
pub struct LoginClient {
    stream: TcpStream,
    version: ProtocolVersion,
    state: ConnectionState,
    bytes_received: usize,
}

impl LoginClient {
    async fn send(&mut self, payload: Vec<u8>) -> std::io::Result<()> {
        let mut framed = Vec::new();
        write_var_int(&mut framed, payload.len() as i32);
        framed.extend_from_slice(&payload);
        self.stream.write_all(&framed).await
    }

    async fn send_kind(&mut self, kind: PacketKind, body: &[u8]) -> std::io::Result<()> {
        let route = PacketRoute::new(self.state, PacketDirection::ServerBound, self.version);
        let Some(id) = route.id_of(kind) else {
            return Err(std::io::Error::other(format!(
                "{kind:?} has no id in {:?} on {}",
                self.state, self.version
            )));
        };

        let mut payload = Vec::new();
        write_var_int(&mut payload, id);
        payload.extend_from_slice(body);
        self.send(payload).await
    }

    async fn read_var_int(&mut self) -> std::io::Result<i32> {
        let mut value = 0_i32;
        for group in 0..5 {
            let mut byte = [0_u8; 1];
            self.stream.read_exact(&mut byte).await?;
            value |= i32::from(byte[0] & 0x7F) << (group * 7);
            if byte[0] & 0x80 == 0 {
                return Ok(value);
            }
        }
        Err(std::io::Error::other("varint longer than five bytes"))
    }

    async fn read_packet(&mut self) -> std::io::Result<Option<PacketKind>> {
        let length = self.read_var_int().await?;
        let length = usize::try_from(length)
            .ok()
            .filter(|size| *size <= MAX_FRAME_LENGTH)
            .ok_or_else(|| std::io::Error::other("implausible frame length"))?;

        let mut frame = vec![0_u8; length];
        self.stream.read_exact(&mut frame).await?;
        self.bytes_received += length;

        let mut id = 0_i32;
        for (group, byte) in frame.iter().take(5).enumerate() {
            id |= i32::from(byte & 0x7F) << (group * 7);
            if byte & 0x80 == 0 {
                break;
            }
        }

        let route = PacketRoute::new(self.state, PacketDirection::ClientBound, self.version);
        Ok(route.kind_of(id))
    }

    fn login_start_body(&self, username: &str) -> Vec<u8> {
        let mut body = Vec::new();
        write_string(&mut body, username);

        // 1.19 and 1.19.1 offer a chat-signing key, which this client declines.
        if (ProtocolVersion::V1_19..=ProtocolVersion::V1_19_1).contains(&self.version) {
            body.push(0);
        }
        if self.version >= ProtocolVersion::V1_20_2 {
            body.extend_from_slice(&[0_u8; 16]);
        } else if self.version >= ProtocolVersion::V1_19_1 {
            body.push(0);
        }
        body
    }

    /// Logs in and returns once the server has put the player in the world.
    ///
    /// The connection is kept open in the returned value: a limbo server's cost is in the
    /// connections it is holding, so letting them close would measure nothing.
    pub async fn join(
        address: SocketAddr,
        version: ProtocolVersion,
        username: &str,
    ) -> std::io::Result<Self> {
        let stream = TcpStream::connect(address).await?;
        let mut client = Self {
            stream,
            version,
            state: ConnectionState::Handshaking,
            bytes_received: 0,
        };

        let mut handshake = Vec::new();
        write_var_int(&mut handshake, version.number());
        write_string(&mut handshake, &address.ip().to_string());
        handshake.extend_from_slice(&address.port().to_be_bytes());
        write_var_int(&mut handshake, 2);
        client.send_kind(PacketKind::Handshake, &handshake).await?;

        client.state = ConnectionState::Login;
        let body = client.login_start_body(username);
        client.send_kind(PacketKind::LoginStart, &body).await?;

        client.walk_to_play().await?;
        Ok(client)
    }

    async fn walk_to_play(&mut self) -> std::io::Result<()> {
        loop {
            let Some(kind) = self.read_packet().await? else {
                continue;
            };

            match (self.state, kind) {
                (ConnectionState::Login, PacketKind::LoginSuccess) => {
                    if self.version >= ProtocolVersion::V1_20_2 {
                        self.send_kind(PacketKind::LoginAcknowledged, &[]).await?;
                        self.state = ConnectionState::Configuration;
                    } else {
                        self.state = ConnectionState::Play;
                        return Ok(());
                    }
                }
                (ConnectionState::Login, PacketKind::LoginDisconnect)
                | (_, PacketKind::Disconnect) => {
                    return Err(std::io::Error::other("the server refused the connection"));
                }
                (ConnectionState::Configuration, PacketKind::KnownPacks) => {
                    self.send_kind(PacketKind::KnownPacks, &[0]).await?;
                }
                (ConnectionState::Configuration, PacketKind::FinishConfiguration) => {
                    self.send_kind(PacketKind::FinishConfiguration, &[]).await?;
                    self.state = ConnectionState::Play;
                    return Ok(());
                }
                _ => {}
            }
        }
    }

    /// Keeps reading until the server stops sending, then reports the total.
    ///
    /// Reaching the play phase is not the end of the join: older clients receive the
    /// whole world burst *after* login success, and newer ones receive most of it during
    /// configuration. Measuring at the moment of arrival would therefore report wildly
    /// different figures for the same work. Waiting for quiet measures the join itself.
    ///
    /// Done for one player rather than all of them, because every player is sent the same
    /// bytes and waiting out a quiet period per player would dominate the run.
    pub async fn drain_join_burst(&mut self, quiet: std::time::Duration) -> usize {
        while tokio::time::timeout(quiet, self.read_packet())
            .await
            .is_ok_and(|read| read.is_ok())
        {
            // Counting happens inside read_packet; this only drives it.
        }
        self.bytes_received
    }
}
