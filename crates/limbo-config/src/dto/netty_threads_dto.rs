use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct NettyThreadsDto {
    /// Kept as an `Option` to tell "not configured" from "configured", because a value
    /// that cannot be honoured is worth a warning and an absent one is not.
    pub boss_group: Option<i32>,
    pub worker_group: i32,
}

impl Default for NettyThreadsDto {
    fn default() -> Self {
        Self {
            boss_group: None,
            worker_group: 4,
        }
    }
}
