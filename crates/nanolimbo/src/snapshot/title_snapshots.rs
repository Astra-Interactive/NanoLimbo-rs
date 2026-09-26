use limbo_packet::PreEncodedPacket;

/// The six title packets, since the three-packet split at 1.17 left the older single
/// packet in place for everyone below it.
pub struct TitleSnapshots {
    pub title: PreEncodedPacket,
    pub subtitle: PreEncodedPacket,
    pub times: PreEncodedPacket,
    pub legacy_title: PreEncodedPacket,
    pub legacy_subtitle: PreEncodedPacket,
    pub legacy_times: PreEncodedPacket,
}
