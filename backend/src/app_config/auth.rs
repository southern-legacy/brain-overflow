use std::collections::HashSet;

use crab_vault::auth::HttpMethod;
use glob::Pattern;
use serde::{Deserialize, Serialize};

use crate::{
    app_config::{
        ConfigItem,
        utils::{
            JwtDecoderConfig, JwtEncoderConfig, RuntimeJwtDecoderConfig, RuntimeJwtEncoderConfig,
        },
    },
    error::fatal::{FatalError, FatalResult, MultiFatalError},
};

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct AuthConfig {
    /// 这里使用 Vec
    ///
    /// 在编译规则时保证如果同一个路径下有多种公开方式时，采取最后指定的公开请求方法而非并集
    #[serde(default)]
    path_rules: Vec<PathRule>,

    #[serde(default)]
    encoder: JwtEncoderConfig,

    /// jwt 鉴权相关设置
    #[serde(default)]
    decoder: JwtDecoderConfig,
}

pub struct RuntimeAuthConfig {
    pub path_rules: Vec<(Pattern, HashSet<HttpMethod>)>,
    pub encoder: RuntimeJwtEncoderConfig,
    pub decoder: RuntimeJwtDecoderConfig,
}

impl TryFrom<AuthConfig> for RuntimeAuthConfig {
    type Error = MultiFatalError;

    fn try_from(
        AuthConfig {
            path_rules,
            encoder,
            decoder,
        }: AuthConfig,
    ) -> FatalResult<Self> {
        let (mut errors, mut compiled_path_rules) = (MultiFatalError::new(), vec![]);

        let encoder = encoder
            .into_runtime()
            .map_err(|mut e| errors.append(&mut e));
        let decoder = decoder
            .into_runtime()
            .map_err(|mut e| errors.append(&mut e));

        for path_rule in path_rules {
            match path_rule.compile() {
                Ok(v) => compiled_path_rules.push(v),
                Err(e) => errors.push(e),
            }
        }

        if let Ok(encoder) = encoder
            && let Ok(decoder) = decoder
        {
            Ok(Self {
                path_rules: compiled_path_rules,
                encoder,
                decoder,
            })
        } else {
            Err(errors)
        }
    }
}

impl ConfigItem for AuthConfig {
    type RuntimeConfig = RuntimeAuthConfig;

    fn into_runtime(self) -> FatalResult<RuntimeAuthConfig> {
        let AuthConfig {
            path_rules,
            encoder,
            decoder,
        } = self;
        let (mut errors, mut compiled_path_rules) = (MultiFatalError::new(), vec![]);

        let encoder = encoder
            .into_runtime()
            .map_err(|mut e| errors.append(&mut e));
        let decoder = decoder
            .into_runtime()
            .map_err(|mut e| errors.append(&mut e));

        for path_rule in path_rules {
            match path_rule.compile() {
                Ok(v) => compiled_path_rules.push(v),
                Err(e) => errors.push(e),
            }
        }

        if let Ok(encoder) = encoder
            && let Ok(decoder) = decoder
        {
            Ok(RuntimeAuthConfig {
                path_rules: compiled_path_rules,
                encoder,
                decoder,
            })
        } else {
            Err(errors)
        }
    }
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
