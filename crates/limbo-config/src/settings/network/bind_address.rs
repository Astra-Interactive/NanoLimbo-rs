use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};

use crate::settings::SettingsError;

/// Turns the `bind` block into the address the listener binds to.
///
/// An absent or empty host means every interface, as the shipped comment promises. A host
/// that is not already an IP literal is resolved here, at startup, because the reference
/// implementation resolved it too — and failing now is better than the reference's
/// behaviour of building an unresolved address and failing at bind time.
pub fn resolve_bind_address(host: Option<&str>, port: u16) -> Result<SocketAddr, SettingsError> {
    let Some(host) = host.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port));
    };

    if let Ok(address) = host.parse::<IpAddr>() {
        return Ok(SocketAddr::new(address, port));
    }

    (host, port)
        .to_socket_addrs()
        .map_err(|error| SettingsError::BindHostUnresolvable {
            host: host.to_owned(),
            source: error,
        })?
        .next()
        .ok_or_else(|| SettingsError::BindHostWithoutAddress {
            host: host.to_owned(),
        })
}
