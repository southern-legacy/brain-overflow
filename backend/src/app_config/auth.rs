use std::collections::HashSet;

use crab_vault::auth::HttpMethod;
use glob::Pattern;
use serde::{Deserialize, Serialize};

use crate::{
    app_config::{
        ConfigItem,
        utils::{
            StaticJwtDecoderConfig, StaticJwtEncoderConfig, JwtDecoderConfig, JwtEncoderConfig,
        },
    },
    error::fatal::{FatalError, FatalResult, MultiFatalError},
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

#[derive(Default, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct PathRule {
    /// 路径的通配符，UNIX shell 通配符
    pattern: String,

    /// 无需 token 即可访问的那些方法
    #[serde(default)]
    public_methods: HashSet<HttpMethod>,
}

impl ConfigItem for StaticAuthConfig {
    type RuntimeConfig = AuthConfig;

    fn into_runtime(self) -> FatalResult<AuthConfig> {
        let StaticAuthConfig {
            encoder,
            decoder,
        } = self;
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

impl From<PathRule> for (String, HashSet<HttpMethod>) {
    fn from(
        PathRule {
            pattern,
            public_methods,
        }: PathRule,
    ) -> (String, HashSet<HttpMethod>) {
        (pattern, public_methods)
    }
}

impl PathRule {
    pub fn compile(&self) -> Result<(Pattern, HashSet<HttpMethod>), FatalError> {
        Pattern::new(&self.pattern)
            .map(|pat| (pat, self.public_methods.iter().copied().collect()))
            .map_err(|e| e.into())
    }
}
