use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use base64::{Engine, prelude::BASE64_STANDARD};
use chrono::TimeDelta;
use clap::error::ErrorKind;
use crab_vault::auth::JwtDecoder;
use crab_vault::auth::{HttpMethod, JwtEncoder};
use glob::Pattern;
use jsonwebtoken::*;
use serde::{Deserialize, Serialize};

use crate::error::cli::{CliError, MultiCliError};

#[derive(Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct AuthConfig {
    /// 这里使用 Vec
    ///
    /// 在编译规则时保证如果同一个路径下有多种公开方式时，采取最后指定的公开请求方法而非并集
    #[serde(default)]
    pub(super) path_rules: Vec<PathRule>,

    #[serde(default)]
    pub(super) crab_vault_encoder_config: JwtEncoderConfig,

    #[serde(default)]
    pub(super) self_encoder_config: JwtEncoderConfig,

    /// jwt 鉴权相关设置
    #[serde(default)]
    pub(super) jwt_decoder_config: JwtDecoderConfig,
}

#[derive(Default, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct PathRule {
    /// 路径的通配符，UNIX shell 通配符
    pub(super) pattern: String,

    /// 无需 token 即可访问的那些方法
    #[serde(default)]
    pub(super) public_methods: HashSet<HttpMethod>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct JwtEncoderConfig {
    encoding_keys: Vec<KeyInfo>,
    issue_as: String,
    audience: Vec<String>,
    expires_in: i64,
    not_valid_in: i64,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct JwtDecoderConfig {
    /// 主键是 issuer，对应的值是 [`KeyInfo`]
    decoding_keys: Vec<(String, KeyInfo)>,
    leeway: u64,
    reject_tokens_expiring_in_less_than: u64,
    audience: Vec<String>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct KeyInfo {
    pub algorithm: Algorithm,
    pub form: KeyForm,

    pub kid: String,

    #[serde(alias = "path")]
    pub key: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
pub enum KeyForm {
    #[default]
    DerInline,
    DerFile,
    PemInline,
    PemFile,
}

impl AuthConfig {
    pub fn encoder_config_to_crab_vault(&self) -> &JwtEncoderConfig {
        &self.crab_vault_encoder_config
    }

    pub fn encoder_to_self(&self) -> &JwtEncoderConfig {
        &self.self_encoder_config
    }

    pub fn decoder(&self) -> &JwtDecoderConfig {
        &self.jwt_decoder_config
    }

    #[must_use]
    pub fn get_compiled_path_rules(&self) -> Vec<(Pattern, HashSet<HttpMethod>)> {
        self.path_rules
            .iter()
            .cloned()
            .filter_map(|rule| rule.compile())
            .collect()
    }
}

impl PartialEq for PathRule {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern
    }
}

impl Eq for PathRule {}

impl Hash for PathRule {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pattern.hash(state);
    }
}

impl PathRule {
    pub fn compile(&self) -> Option<(Pattern, HashSet<HttpMethod>)> {
        match Pattern::new(&self.pattern) {
            Ok(val) => Some((val, self.public_methods.iter().copied().collect())),
            Err(e) => {
                tracing::error!(
                    "the PATH `{}` of path rules is not written in valid UNIX shell format, so this pattern is skipped, if that matters, please check your configuration file, details: {e}",
                    self.pattern
                );
                None
            }
        }
    }
}

impl TryFrom<JwtEncoderConfig> for JwtEncoder {
    type Error = MultiCliError;

    fn try_from(
        JwtEncoderConfig {
            encoding_keys,
            issue_as: _,
            audience: _,
            expires_in: _,
            not_valid_in: _,
        }: JwtEncoderConfig,
    ) -> Result<Self, Self::Error> {
        let mut mapping = HashMap::new();
        let mut errors = MultiCliError::new();

        for key in encoding_keys {
            match key.build_as_encode_key() {
                Ok((kid, alg, key)) => {
                    mapping.insert(kid, (key, alg));
                }
                Err(e) => {
                    errors.add(e);
                }
            }
        }

        if mapping.is_empty() {
            errors.add(CliError::new(
                ErrorKind::Io,
                "you should feed me at least one kid, encoding key pair".to_string(),
                None,
            ));
        }

        if errors.is_empty() {
            Ok(JwtEncoder::new(mapping))
        } else {
            Err(errors)
        }
    }
}

impl JwtEncoderConfig {
    pub fn algorithms(&self) -> Vec<Algorithm> {
        self.encoding_keys.iter().map(|key| key.algorithm).collect()
    }

    pub fn kids(&self) -> Vec<String> {
        self.encoding_keys
            .iter()
            .map(|key| &key.kid)
            .cloned()
            .collect()
    }

    pub fn issue_as(&self) -> &str {
        &self.issue_as
    }

    pub fn audience(&self) -> &[String] {
        self.audience.as_slice()
    }

    pub fn expire_in(&self) -> TimeDelta {
        TimeDelta::new(self.expires_in, 0).unwrap()
    }

    pub fn not_valid_in(&self) -> TimeDelta {
        TimeDelta::new(self.expires_in, 0).unwrap()
    }
}

impl TryFrom<JwtDecoderConfig> for JwtDecoder {
    type Error = MultiCliError;

    fn try_from(
        JwtDecoderConfig {
            decoding_keys,
            leeway,
            reject_tokens_expiring_in_less_than,
            audience: aud,
        }: JwtDecoderConfig,
    ) -> Result<Self, Self::Error> {
        let mut mapping = HashMap::new();
        let mut errors = MultiCliError::new();
        let mut algorithms = vec![];
        let mut authorized_issuers = vec![];

        for (iss, key) in decoding_keys {
            match key.build_as_decode_key() {
                Ok((kid, alg, key)) => {
                    authorized_issuers.push(iss.clone());
                    algorithms.push(alg);
                    mapping.insert((iss, kid), key);
                }
                Err(e) => {
                    errors.add(e);
                }
            }
        }

        if mapping.is_empty() {
            errors.add(CliError::new(
                ErrorKind::Io,
                "you should feed me at least one kid, encoding key pair".to_string(),
                None,
            ));
        }

        if errors.is_empty() {
            Ok(
                JwtDecoder::new(mapping, &algorithms, &authorized_issuers, &aud)
                    .reject_tokens_expiring_in_less_than(reject_tokens_expiring_in_less_than)
                    .leeway(leeway),
            )
        } else {
            Err(errors)
        }
    }
}

impl KeyInfo {
    fn get_key(&self) -> Result<Vec<u8>, CliError> {
        let res = match self.form {
            KeyForm::DerInline => BASE64_STANDARD.decode(self.key.clone()).map_err(|e| {
                CliError::from(e).add_source(format!(
                    "while decoding the given jwt secrete key `{}` into binary form",
                    self.key
                        .get(0..4)
                        .map(|val| format!("{val}..."))
                        .unwrap_or(self.key.clone())
                ))
            })?,
            KeyForm::DerFile => std::fs::read(&self.key).map_err(|e| {
                CliError::from(e).add_source(format!("while reading the der key from {}", self.key))
            })?,
            KeyForm::PemInline => self.key.clone().into_bytes(),
            KeyForm::PemFile => std::fs::read(&self.key).map_err(|e| {
                CliError::from(e).add_source(format!("while reading the pem key from {}", self.key))
            })?,
        };

        if res.len() < 32 {
            tracing::warn!(
                "the secret key `{}` is too short to prevent brute cracking",
                self.key
                    .get(0..4)
                    .map(|val| format!("{val}..."))
                    .unwrap_or(self.key.clone())
            )
        }

        Ok(res)
    }

    fn build_as_encode_key(&self) -> Result<(String, Algorithm, EncodingKey), CliError> {
        if self.form.is_der() {
            let build_from_der = match self.algorithm {
                Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => EncodingKey::from_secret,
                Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 => EncodingKey::from_rsa_der,
                Algorithm::PS256 | Algorithm::PS384 | Algorithm::PS512 => EncodingKey::from_rsa_der,
                Algorithm::ES256 | Algorithm::ES384 => EncodingKey::from_ec_der,
                Algorithm::EdDSA => EncodingKey::from_ed_der,
            };

            Ok((
                self.kid.clone(),
                self.algorithm,
                build_from_der(&self.get_key().map_err(|e| {
                    e.add_source("while building jwt encoding key from a der form".into())
                })?),
            ))
        } else if self.form.is_pem() {
            let build_from_pem = match self.algorithm {
                Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                    return Err(CliError::new(
                        ErrorKind::Io,
                        format!(
                            "cannot use a .pem file `{}` to store a hmac secret key, you should use a .der file or a inline base64 encoded string",
                            self.key
                        ),
                        Some("while building jwt encoding key from a der form".into()),
                    ));
                }
                Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 => EncodingKey::from_rsa_pem,
                Algorithm::PS256 | Algorithm::PS384 | Algorithm::PS512 => EncodingKey::from_rsa_pem,
                Algorithm::ES256 | Algorithm::ES384 => EncodingKey::from_ec_pem,
                Algorithm::EdDSA => EncodingKey::from_ed_pem,
            };

            Ok((
                self.kid.clone(),
                self.algorithm,
                build_from_pem(&self.get_key().map_err(|e| {
                    e.add_source("while building jwt encoding key from a pem form".into())
                })?)
                .map_err(|e| {
                    CliError::new(
                        ErrorKind::Io,
                        e.to_string(),
                        Some("while building jwt encoding key from a pem form".into()),
                    )
                })?,
            ))
        } else {
            unreachable!(
                "Sylvan, 你加了新的变体但是没有添加相应的条件判断，去检查你的 is_der 和 is_pem 方法是否包含了所有的情况"
            )
        }
    }

    fn build_as_decode_key(&self) -> Result<(String, Algorithm, DecodingKey), CliError> {
        if self.form.is_der() {
            let build_from_der = match self.algorithm {
                Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => DecodingKey::from_secret,
                Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 => DecodingKey::from_rsa_der,
                Algorithm::PS256 | Algorithm::PS384 | Algorithm::PS512 => DecodingKey::from_rsa_der,
                Algorithm::ES256 | Algorithm::ES384 => DecodingKey::from_ec_der,
                Algorithm::EdDSA => DecodingKey::from_ed_der,
            };

            Ok((
                self.kid.clone(),
                self.algorithm,
                build_from_der(&self.get_key().map_err(|e| {
                    e.add_source("while building jwt encoding key from a der form".into())
                })?),
            ))
        } else if self.form.is_pem() {
            let build_from_pem = match self.algorithm {
                Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                    return Err(CliError::new(
                        ErrorKind::Io,
                        format!(
                            "cannot use a .pem file `{}` to store a hmac secret key, you should use a .der file or a inline base64 encoded string",
                            self.key
                        ),
                        Some("while building jwt decoding key from a pem form".into()),
                    ));
                }
                Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 => DecodingKey::from_rsa_pem,
                Algorithm::PS256 | Algorithm::PS384 | Algorithm::PS512 => DecodingKey::from_rsa_pem,
                Algorithm::ES256 | Algorithm::ES384 => DecodingKey::from_ec_pem,
                Algorithm::EdDSA => DecodingKey::from_ed_pem,
            };

            Ok((
                self.kid.clone(),
                self.algorithm,
                build_from_pem(&self.get_key()?).map_err(|e| {
                    CliError::new(
                        ErrorKind::Io,
                        e.to_string(),
                        Some("while building jwt decoding key from a pem form".into()),
                    )
                })?,
            ))
        } else {
            unreachable!(
                "Sylvan, 你加了新的变体但是没有添加相应的条件判断，去检查你的 is_der 和 is_pem 方法是否包含了所有的情况"
            )
        }
    }
}

impl KeyForm {
    fn is_der(&self) -> bool {
        matches!(self, KeyForm::DerInline | KeyForm::DerFile)
    }

    fn is_pem(&self) -> bool {
        matches!(self, KeyForm::PemInline | KeyForm::PemFile)
    }
}
