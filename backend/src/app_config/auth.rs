use serde::{Deserialize, Serialize};

use crate::{
    app_config::{
        ConfigItem,
        util::{
            JwtDecoderConfig, JwtEncoderConfig, StaticJwtDecoderConfig, StaticJwtEncoderConfig,
        },
    },
    error::fatal::{FatalResult, MultiFatalError},
};

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields, default)]
pub(super) struct StaticAuthConfig {
    #[serde(default)]
    encoder: StaticJwtEncoderConfig,

    /// jwt 鉴权相关设置
    #[serde(default)]
    decoder: StaticJwtDecoderConfig,
}

pub struct AuthConfig {
    pub encoder_config: JwtEncoderConfig,
    pub decoder_config: JwtDecoderConfig,
}

impl ConfigItem for StaticAuthConfig {
    type RuntimeConfig = AuthConfig;

    fn into_runtime(self) -> FatalResult<AuthConfig> {
        let StaticAuthConfig { encoder, decoder } = self;
        let mut errors = MultiFatalError::new();

        let encoder = encoder
            .into_runtime()
            .map_err(|mut e| errors.append(&mut e));
        let decoder = decoder
            .into_runtime()
            .map_err(|mut e| errors.append(&mut e));

        if let Ok(encoder) = encoder
            && let Ok(decoder) = decoder
        {
            Ok(AuthConfig {
                encoder_config: encoder,
                decoder_config: decoder,
            })
        } else {
            Err(errors)
        }
    }
}
