use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};

use crate::settings_error::SettingsError;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_no_host_when_resolved_then_it_binds_every_interface() {
        let address = resolve_bind_address(None, 25565).unwrap();

        assert!(address.ip().is_unspecified());
        assert_eq!(address.port(), 25565);
    }

    #[test]
    fn given_an_empty_host_when_resolved_then_it_binds_every_interface() {
        let address = resolve_bind_address(Some(""), 25565).unwrap();

        assert!(address.ip().is_unspecified());
    }

    #[test]
    fn given_an_ipv4_literal_when_resolved_then_it_is_used_as_written() {
        let address = resolve_bind_address(Some("127.0.0.1"), 25565).unwrap();

        assert_eq!(address, SocketAddr::from(([127, 0, 0, 1], 25565)));
    }

    #[test]
    fn given_an_ipv6_literal_when_resolved_then_it_is_used_as_written() {
        let address = resolve_bind_address(Some("::1"), 25565).unwrap();

        assert!(address.is_ipv6());
        assert!(address.ip().is_loopback());
    }

    #[test]
    fn given_a_host_name_when_resolved_then_it_becomes_an_address() {
        let address = resolve_bind_address(Some("localhost"), 65535).unwrap();

        assert!(address.ip().is_loopback());
        assert_eq!(address.port(), 65535);
    }

    #[test]
    fn given_a_host_that_cannot_exist_when_resolved_then_it_is_an_error_rather_than_a_panic() {
        let resolved = resolve_bind_address(Some("host.invalid"), 25565);

        assert!(resolved.is_err());
    }
}
