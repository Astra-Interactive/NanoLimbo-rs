use serde::Deserialize;

use crate::settings::schema::NettyThreadsDto;
use crate::settings::schema::ScalarText;

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct NettyDto {
    pub transport_type: Option<ScalarText>,
    pub threads: NettyThreadsDto,
}
