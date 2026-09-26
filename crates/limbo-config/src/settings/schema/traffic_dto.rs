use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TrafficDto {
    pub enable: bool,
    /// Bytes. Negative means unlimited.
    pub max_packet_size: i32,
    /// Seconds. Negative means the rates below are not measured.
    pub interval: f64,
    pub max_packet_rate: f64,
    pub max_packet_bytes_rate: f64,
}

impl Default for TrafficDto {
    fn default() -> Self {
        Self {
            enable: false,
            max_packet_size: -1,
            interval: -1.0,
            max_packet_rate: -1.0,
            max_packet_bytes_rate: -1.0,
        }
    }
}
