mod forwarding {
    mod bungee_guard_verifier;
    mod handshake_fields;
    mod legacy_forwarding;
    mod modern_forwarding_verifier;
}

mod frame {
    mod length_prefix;
    mod var_int_frame_codec;
}

mod identity {
    mod offline_uuid;
    mod uuid_text;
}

mod traffic {
    mod packet_bucket;
    mod traffic_limiter;
    mod traffic_limits;
}
