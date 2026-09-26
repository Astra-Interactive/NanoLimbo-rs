//! Clientbound packets: what the server says, in the shape each protocol version reads.
//!
//! Every packet here is a plain description of its payload with public fields, and every
//! encoder is a transcription of the Java implementation's `encode` method including its
//! version conditionals. Payloads carry no id prefix — the id depends on the connection
//! state as well as the version, so it is resolved through
//! [`PacketRoute`](limbo_protocol::packet::PacketRoute) by the layer that frames them.
//!
//! Nothing in this crate reads a clock or a random source. Values the Java implementation
//! drew from `Random` — entity ids, teleport ids, session and boss bar uuids, keep alive
//! ids — arrive as fields, which is what makes byte-level golden tests possible.

pub mod configuration {
    //! Packets sent while the connection is in the configuration state, which 1.20.2 added
    //! between login and play.

    mod finish_configuration;
    mod known_pack;
    mod known_packs;
    mod registry_data;
    mod registry_entry;
    mod update_tags;

    pub use finish_configuration::FinishConfiguration;
    pub use known_pack::KnownPack;
    pub use known_packs::KnownPacks;
    pub use registry_data::RegistryData;
    pub use registry_entry::RegistryEntry;
    pub use update_tags::UpdateTags;
}

pub mod login {
    //! Packets sent while the connection is in the login state.

    mod login_disconnect;
    mod login_plugin_request;
    mod login_success;

    pub use login_disconnect::LoginDisconnect;
    pub use login_plugin_request::LoginPluginRequest;
    pub use login_success::LoginSuccess;
}

pub mod play {
    //! Packets sent while the connection is in the play state.

    mod boss_bar;
    mod boss_bar_color;
    mod boss_bar_division;
    mod chat_message;
    mod chat_position;
    mod chunk_with_light;
    mod declare_commands;
    mod disconnect;
    mod game_event;
    mod join_game;
    mod keep_alive;
    mod player_abilities;
    mod player_info;
    mod player_list_header;
    mod player_position_and_look;
    mod plugin_message;
    mod spawn_position;
    mod title_legacy;
    mod title_set_subtitle;
    mod title_set_title;
    mod title_times;

    pub use boss_bar::BossBar;
    pub use boss_bar_color::BossBarColor;
    pub use boss_bar_division::BossBarDivision;
    pub use chat_message::ChatMessage;
    pub use chat_position::ChatPosition;
    pub use chunk_with_light::ChunkWithLight;
    pub use declare_commands::DeclareCommands;
    pub use disconnect::Disconnect;
    pub use game_event::GameEvent;
    pub use join_game::JoinGame;
    pub use keep_alive::KeepAlive;
    pub use player_abilities::PlayerAbilities;
    pub use player_info::PlayerInfo;
    pub use player_list_header::PlayerListHeader;
    pub use player_position_and_look::PlayerPositionAndLook;
    pub use plugin_message::PluginMessage;
    pub use spawn_position::SpawnPosition;
    pub use title_legacy::TitleLegacy;
    pub use title_set_subtitle::TitleSetSubTitle;
    pub use title_set_title::TitleSetTitle;
    pub use title_times::TitleTimes;
}

pub mod status {
    //! Packets sent while the connection is in the status state, answering a server ping.

    mod status_player;
    mod status_response;

    pub use status_player::StatusPlayer;
    pub use status_response::StatusResponse;
}

mod encoding {
    //! How a packet becomes bytes: the trait every packet implements, the one way encoding
    //! can fail, and the per-version payload cache built from it at startup.

    mod clientbound_packet;
    mod packet_encode_error;
    mod pre_encoded_packet;
    mod write_bool;

    pub use clientbound_packet::ClientboundPacket;
    pub use packet_encode_error::PacketEncodeError;
    pub use pre_encoded_packet::PreEncodedPacket;
    pub(crate) use write_bool::write_bool;
}

pub use encoding::{ClientboundPacket, PacketEncodeError, PreEncodedPacket};
