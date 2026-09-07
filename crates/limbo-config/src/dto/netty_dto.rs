use serde::Deserialize;

use crate::dto::netty_threads_dto::NettyThreadsDto;
use crate::dto::scalar_text::ScalarText;

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct NettyDto {
    pub transport_type: Option<ScalarText>,
    pub threads: NettyThreadsDto,
}
