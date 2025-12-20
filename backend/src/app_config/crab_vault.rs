use serde::{Deserialize, Serialize};

use crate::{
    app_config::{
        ConfigItem,
        utils::{StaticJwtEncoderConfig, JwtEncoderConfig},
    },
    error::fatal::FatalResult,
};

#[derive(Serialize, Deserialize, Clone, Default)]
pub(super) struct StaticCrabVaultConfig {
    encoder: StaticJwtEncoderConfig,
}

pub struct CrabVaultConfig {
    pub encoder_config: JwtEncoderConfig,
}

impl ConfigItem for StaticCrabVaultConfig {
    type RuntimeConfig = CrabVaultConfig;

    fn into_runtime(self) -> FatalResult<CrabVaultConfig> {
        let StaticCrabVaultConfig { encoder } = self;
        Ok(CrabVaultConfig {
            encoder_config: encoder.into_runtime()?,
        })
    }
}
