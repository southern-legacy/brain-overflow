use serde::{Deserialize, Serialize};

use crate::{
    app_config::{
        ConfigItem,
        utils::{JwtEncoderConfig, RuntimeJwtEncoderConfig},
    },
    error::fatal::FatalResult,
};

#[derive(Serialize, Deserialize, Clone, Default)]
pub(super) struct CrabVaultConfig {
    encoder: JwtEncoderConfig,
}

pub struct RuntimeCrabVaultConfig {
    pub encoder: RuntimeJwtEncoderConfig,
}

impl ConfigItem for CrabVaultConfig {
    type RuntimeConfig = RuntimeCrabVaultConfig;

    fn into_runtime(self) -> FatalResult<RuntimeCrabVaultConfig> {
        let CrabVaultConfig { encoder } = self;
        Ok(RuntimeCrabVaultConfig {
            encoder: encoder.into_runtime()?,
        })
    }
}
