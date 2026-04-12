use auth::JwtDecoder;
use serde::{Deserialize, Serialize};

use crate::{
    app_config::{
        ConfigItem,
        util::{JwtEncoderConfig, StaticJwtDecoderConfig, StaticJwtEncoderConfig},
    },
    error::fatal::{FatalResult, MultiFatalError},
};

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(super) struct StaticAuthConfig {
    access: StaticJwtEncoderConfig,
    refresh: StaticJwtEncoderConfig,

    /// jwt 鉴权相关设置
    decoder: StaticJwtDecoderConfig,
}

pub struct AuthConfig {
    pub access: JwtEncoderConfig,
    pub refresh: JwtEncoderConfig,
    pub decoder: JwtDecoder,
}

impl ConfigItem for StaticAuthConfig {
    type RuntimeConfig = AuthConfig;

    fn into_runtime(self) -> FatalResult<AuthConfig> {
        let StaticAuthConfig {
            access,
            refresh,
            decoder,
        } = self;
        let mut errors = MultiFatalError::new();

        let access = access.into_runtime().map_err(|mut e| errors.append(&mut e));
        let refresh = refresh.into_runtime().map_err(|mut e| errors.append(&mut e));
        let decoder = decoder.into_runtime().map_err(|mut e| errors.append(&mut e));

        if let Ok(access) = access
            && let Ok(refresh) = refresh
            && let Ok(decoder) = decoder
        {
            Ok(AuthConfig {
                access,
                refresh,
                decoder,
            })
        } else {
            Err(errors)
        }
    }
}
