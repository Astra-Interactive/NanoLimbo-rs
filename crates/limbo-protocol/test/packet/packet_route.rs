use std::collections::HashMap;

use crate::packet::packet_mappings;
use crate::packet::{ConnectionState, PacketDirection, PacketKind, PacketRoute};
use crate::version::ProtocolVersion;

fn every_route() -> impl Iterator<Item = PacketRoute> {
    ConnectionState::ALL.into_iter().flat_map(|state| {
        PacketDirection::ALL.into_iter().flat_map(move |direction| {
            ProtocolVersion::all().map(move |version| PacketRoute::new(state, direction, version))
        })
    })
}

#[test]
fn given_any_route_when_mappings_are_collected_then_each_id_names_one_packet() {
    for route in every_route() {
        let mut claimed: HashMap<i32, PacketKind> = HashMap::new();

        for mapping in route.mappings() {
            if let Some(previous) = claimed.insert(mapping.id(), mapping.kind()) {
                assert_eq!(
                    previous,
                    mapping.kind(),
                    "{:?}/{:?} on {}: id {:#04X} maps to both {previous:?} and {:?}",
                    route.state(),
                    route.direction(),
                    route.version(),
                    mapping.id(),
                    mapping.kind(),
                );
            }
        }
    }
}

#[test]
fn given_any_route_when_mappings_are_collected_then_each_packet_has_one_id() {
    for route in every_route() {
        let mut assigned: HashMap<PacketKind, i32> = HashMap::new();

        for mapping in route.mappings() {
            if let Some(previous) = assigned.insert(mapping.kind(), mapping.id()) {
                assert_eq!(
                    previous,
                    mapping.id(),
                    "{:?}/{:?} on {}: {:?} maps to both {previous:#04X} and {:#04X}",
                    route.state(),
                    route.direction(),
                    route.version(),
                    mapping.kind(),
                    mapping.id(),
                );
            }
        }
    }
}

#[test]
fn given_the_tables_when_scanned_then_no_version_range_is_inverted() {
    for state in ConnectionState::ALL {
        for direction in PacketDirection::ALL {
            for mapping in packet_mappings::table(state, direction) {
                let range = mapping.versions();

                assert!(
                    range.start() <= range.end(),
                    "{state:?}/{direction:?} {:?}: range {}..{} is inverted",
                    mapping.kind(),
                    range.start(),
                    range.end(),
                );
            }
        }
    }
}

#[test]
fn given_a_mapped_packet_when_looked_up_both_ways_then_the_lookups_agree() {
    for route in every_route() {
        for mapping in route.mappings() {
            assert_eq!(route.kind_of(mapping.id()), Some(mapping.kind()));
            assert_eq!(route.id_of(mapping.kind()), Some(mapping.id()));
        }
    }
}

#[test]
fn given_an_id_absent_from_the_route_when_looked_up_then_nothing_is_returned() {
    let route = PacketRoute::new(
        ConnectionState::Handshaking,
        PacketDirection::ServerBound,
        ProtocolVersion::V1_20_2,
    );

    assert_eq!(route.kind_of(0x00), Some(PacketKind::Handshake));
    assert_eq!(route.kind_of(0x01), None);
}

#[test]
fn given_configuration_serverbound_when_resolved_then_plugin_message_shifts_at_1_20_5() {
    let route_of = |version| {
        PacketRoute::new(
            ConnectionState::Configuration,
            PacketDirection::ServerBound,
            version,
        )
    };

    // Cross-checked against PrismarineJS/minecraft-data. The Java table put
    // PluginMessage at 0x02 from 1.20.2, where 0x02 is FinishConfiguration.
    for version in [ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_3] {
        assert_eq!(
            route_of(version).kind_of(0x01),
            Some(PacketKind::PluginMessage)
        );
        assert_eq!(
            route_of(version).kind_of(0x02),
            Some(PacketKind::FinishConfiguration)
        );
    }

    for version in [ProtocolVersion::V1_20_5, ProtocolVersion::V1_21_11] {
        assert_eq!(
            route_of(version).kind_of(0x02),
            Some(PacketKind::PluginMessage)
        );
        assert_eq!(
            route_of(version).kind_of(0x03),
            Some(PacketKind::FinishConfiguration)
        );
        assert_eq!(
            route_of(version).kind_of(0x07),
            Some(PacketKind::KnownPacks)
        );
    }
}
