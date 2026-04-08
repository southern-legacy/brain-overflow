use serde::{Deserialize, Serialize};

use crate::{
    app_config::{
        ConfigItem,
        util::{JwtEncoderConfig, StaticJwtEncoderConfig},
    },
    error::fatal::FatalResult,
};

#[derive(Serialize, Deserialize, Clone, Default)]
pub(super) struct StaticS3Config {
    encoder: StaticJwtEncoderConfig,
    location: String,
}

pub struct S3Config {
    pub encoder_config: JwtEncoderConfig,
    pub location: String,
}

impl ConfigItem for StaticS3Config {
    type RuntimeConfig = S3Config;

    fn into_runtime(self) -> FatalResult<S3Config> {
        let StaticS3Config { encoder, location } = self;
        Ok(S3Config {
            encoder_config: encoder.into_runtime()?,
            location,
        })
    }
}

impl S3Config {
    /// # 获取 key 对应的 url（包含域名）
    /// 实现非常简单，就是一个 `format`
    pub fn location_of_asset(&self, key: &str) -> String {
        format!("{}{}", self.location, key)
    }
}
