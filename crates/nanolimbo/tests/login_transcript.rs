//! Walks every supported protocol version through a real server, over a real socket.
//!
//! The packet-level suites prove each packet's bytes; this proves the sequence — that the
//! server sends the right things, in the right order, for the phase the client is in. The
//! version branching it covers is exactly where a limbo server goes wrong: the
//! configuration phase appearing at 1.20.2, known packs at 1.20.5, and the older clients
//! that get neither.
//!
//! It runs against the shipped configuration, so it also proves that configuration
//! actually serves players rather than merely parsing.

#![allow(clippy::expect_used, clippy::panic, clippy::indexing_slicing)]

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use limbo_protocol::packet::{ConnectionState, PacketDirection, PacketKind, PacketRoute};
use limbo_protocol::version::ProtocolVersion;
use nanolimbo::listener::accept_until_shutdown;
use nanolimbo::startup::prepare;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;

/// Long enough that a loaded machine does not fail the test, short enough that a genuine
/// hang is reported rather than waited on.
const REPLY_TIMEOUT: Duration = Duration::from_secs(10);

/// How long to wait for another packet before deciding the join burst is over.
///
/// Deliberately far below the five-second keep alive interval. An earlier version of this
/// test stopped reading when it saw a keep alive, which the periodic timer supplies on its
/// own — so deleting the keep alive from the join burst still passed, just eighty times
/// slower. Quiescence measures the burst itself.
const QUIET_PERIOD: Duration = Duration::from_millis(400);

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

/// A client that speaks just enough of the protocol to log in.
struct FakeClient {
    stream: TcpStream,
    version: ProtocolVersion,
    state: ConnectionState,
}

impl FakeClient {
    async fn connect(address: SocketAddr, version: ProtocolVersion) -> Self {
        let stream = TcpStream::connect(address)
            .await
            .expect("the server must accept a connection");
        Self {
            stream,
            version,
            state: ConnectionState::Handshaking,
        }
    }

    async fn send(&mut self, payload: Vec<u8>) {
        let mut framed = Vec::new();
        write_var_int(&mut framed, payload.len() as i32);
        framed.extend_from_slice(&payload);
        self.stream
            .write_all(&framed)
            .await
            .expect("the server must still be reading");
    }

    /// Sends a packet by name, so the test never hard-codes an id the tables own.
    async fn send_kind(&mut self, kind: PacketKind, body: &[u8]) {
        let route = PacketRoute::new(self.state, PacketDirection::ServerBound, self.version);
        let id = route
            .id_of(kind)
            .unwrap_or_else(|| panic!("{kind:?} has no id in {:?}/{}", self.state, self.version));

        let mut payload = Vec::new();
        write_var_int(&mut payload, id);
        payload.extend_from_slice(body);
        self.send(payload).await;
    }

    async fn read_byte(&mut self) -> Option<u8> {
        let mut byte = [0_u8; 1];
        match self.stream.read_exact(&mut byte).await {
            Ok(_) => Some(byte[0]),
            Err(_closed) => None,
        }
    }

    async fn read_var_int(&mut self) -> Option<i32> {
        let mut value = 0_i32;
        for group in 0..5 {
            let byte = self.read_byte().await?;
            value |= i32::from(byte & 0x7F) << (group * 7);
            if byte & 0x80 == 0 {
                return Some(value);
            }
        }
        None
    }

    /// Reads one packet, giving up once the server has gone quiet.
    async fn read_packet(&mut self) -> Option<PacketKind> {
        let length = tokio::time::timeout(QUIET_PERIOD, self.read_var_int())
            .await
            .ok()??;
        let mut frame = vec![0_u8; usize::try_from(length).ok()?];
        self.stream.read_exact(&mut frame).await.ok()?;

        let mut cursor = frame.as_slice();
        let mut id = 0_i32;
        for group in 0..5 {
            let (byte, rest) = cursor.split_first()?;
            cursor = rest;
            id |= i32::from(byte & 0x7F) << (group * 7);
            if byte & 0x80 == 0 {
                break;
            }
        }

        let route = PacketRoute::new(self.state, PacketDirection::ClientBound, self.version);
        route.kind_of(id)
    }

    fn login_start_body(&self) -> Vec<u8> {
        let mut body = Vec::new();
        write_string(&mut body, "TestPlayer");

        // 1.19 and 1.19.1 carry a chat-signing key, which we decline to provide.
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

    /// Drives a full join and returns the packets the server sent in the play phase.
    async fn join(&mut self, port: u16) -> Vec<PacketKind> {
        let mut handshake = Vec::new();
        write_var_int(&mut handshake, self.version.number());
        write_string(&mut handshake, "localhost");
        handshake.extend_from_slice(&port.to_be_bytes());
        write_var_int(&mut handshake, 2);
        self.send_kind(PacketKind::Handshake, &handshake).await;

        self.state = ConnectionState::Login;
        let body = self.login_start_body();
        self.send_kind(PacketKind::LoginStart, &body).await;

        let mut in_play = Vec::new();
        while let Some(kind) = self.read_packet().await {
            match (self.state, kind) {
                (ConnectionState::Login, PacketKind::LoginSuccess) => {
                    if self.version >= ProtocolVersion::V1_20_2 {
                        self.send_kind(PacketKind::LoginAcknowledged, &[]).await;
                        self.state = ConnectionState::Configuration;
                    } else {
                        self.state = ConnectionState::Play;
                    }
                }
                (ConnectionState::Configuration, PacketKind::KnownPacks) => {
                    self.send_kind(PacketKind::KnownPacks, &[0]).await;
                }
                (ConnectionState::Configuration, PacketKind::FinishConfiguration) => {
                    self.send_kind(PacketKind::FinishConfiguration, &[]).await;
                    self.state = ConnectionState::Play;
                }
                (ConnectionState::Play, seen) => in_play.push(seen),
                _ => {}
            }
        }
        in_play
    }
}

/// Starts a server on an ephemeral port using the shipped configuration.
async fn start_server() -> SocketAddr {
    let root = std::env::temp_dir().join(format!("nanolimbo-transcript-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("a working directory");

    let prepared = prepare(&root).expect("the shipped configuration must start a server");
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
        .await
        .expect("an ephemeral port");
    let address = listener.local_addr().expect("the bound address");

    let (shutdown, _) = broadcast::channel(1);
    tokio::spawn(accept_until_shutdown(
        listener,
        Arc::clone(&prepared.context),
        shutdown,
    ));

    address
}

#[tokio::test(flavor = "multi_thread")]
async fn given_every_supported_version_when_a_client_joins_then_it_reaches_the_play_phase() {
    let address = start_server().await;

    // Run the versions together rather than one after another. Each client waits out a
    // quiet period to know the burst has ended, and fifty-one of those in sequence is
    // most of a minute of doing nothing.
    let joins: Vec<_> = ProtocolVersion::all()
        .map(|version| {
            tokio::spawn(async move {
                let mut client = FakeClient::connect(address, version).await;
                let seen = tokio::time::timeout(REPLY_TIMEOUT, client.join(address.port()))
                    .await
                    .unwrap_or_else(|_| panic!("{version} never finished joining"));
                (version, seen)
            })
        })
        .collect();

    let mut checked = 0;
    for join in joins {
        let (version, seen) = join.await.expect("a client task must not panic");

        assert!(
            seen.contains(&PacketKind::JoinGame),
            "{version} never received the join game packet, saw {seen:?}"
        );
        assert!(
            seen.contains(&PacketKind::PlayerPositionAndLook),
            "{version} was never placed anywhere, saw {seen:?}"
        );
        assert!(
            seen.contains(&PacketKind::KeepAlive),
            "{version} was never asked to keep the connection alive, saw {seen:?}"
        );
        checked += 1;
    }

    assert_eq!(
        checked,
        ProtocolVersion::all().count(),
        "every supported version must be exercised"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn given_a_modern_client_when_it_joins_then_the_configuration_phase_precedes_play() {
    let address = start_server().await;

    let mut client = FakeClient::connect(address, ProtocolVersion::V1_21).await;
    tokio::time::timeout(REPLY_TIMEOUT, client.join(address.port()))
        .await
        .expect("joining must not hang");

    assert_eq!(client.state, ConnectionState::Play);
}

#[tokio::test(flavor = "multi_thread")]
async fn given_a_client_predating_the_configuration_phase_then_login_leads_straight_to_play() {
    let address = start_server().await;

    let mut client = FakeClient::connect(address, ProtocolVersion::V1_8).await;
    let seen = tokio::time::timeout(REPLY_TIMEOUT, client.join(address.port()))
        .await
        .expect("joining must not hang");

    assert!(seen.contains(&PacketKind::JoinGame));
    assert!(
        !seen.contains(&PacketKind::RegistryData),
        "a 1.8 client must never be sent registry data"
    );
}
