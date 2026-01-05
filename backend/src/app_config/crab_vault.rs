use serde::{Deserialize, Serialize};

use crate::{
    app_config::{
        ConfigItem,
        util::{JwtEncoderConfig, StaticJwtEncoderConfig},
    },
    error::fatal::FatalResult,
};

#[derive(Serialize, Deserialize, Clone, Default)]
pub(super) struct StaticCrabVaultConfig {
    encoder: StaticJwtEncoderConfig,
    location: String,
}

pub struct CrabVaultConfig {
    pub encoder_config: JwtEncoderConfig,
    pub location: String,
}

impl ConfigItem for StaticCrabVaultConfig {
    type RuntimeConfig = CrabVaultConfig;

    fn into_runtime(self) -> FatalResult<CrabVaultConfig> {
        let StaticCrabVaultConfig { encoder, location } = self;
        Ok(CrabVaultConfig {
            encoder_config: encoder.into_runtime()?,
            location,
        })
    }
}

impl CrabVaultConfig {
    /// # 获取 key 对应的 url（包含域名）
    /// 实现非常简单，就是一个 `format`
    pub fn location_of_asset(&self, key: &str) -> String {
        format!("{}{}", self.location, key)
    }
}
