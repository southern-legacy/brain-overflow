use base64::{Engine, prelude::BASE64_STANDARD};
use chrono::Duration;
use clap::error::ErrorKind;
use crab_vault_auth::JwtConfig;
use jsonwebtoken::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::cli::{CliError, CliResult, MultiCliError};

/// 这个就是配置文件的直接映射
#[derive(Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct JwtConfigBuilder {
    encoding: AlgKeyPair,
    decoding: Vec<AlgKeyPair>,
    validation: ValidationConfig,

    #[serde(default)]
    iss_config: IssueConfig,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct IssueConfig {
    /// 签发令牌的时候署什么名，即令牌中的 iss 字段是什么值
    issue_as: Option<String>,

    /// 签发令牌的时候这个令牌的受众是谁
    issue_to: Option<Vec<String>>,

    expire_in: Option<Duration>,
    not_before: Option<Duration>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct AlgKeyPair {
    algorithm: Algorithm,
    form: KeyForm,

    #[serde(alias = "path")]
    key: String,
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

/// [`jsonwebtoken::Validation`] 没有实现 [`Deserialize`]，
///
/// 不能通过配置文件读取，于是就搞了这个中间层
#[derive(Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ValidationConfig {
    required_spec_claims: Vec<String>,
    leeway: u64,
    reject_tokens_expiring_in_less_than: u64,
    validate_exp: bool,
    validate_nbf: bool,
    accepted_aud: Option<Vec<String>>,
    accepted_iss: Option<Vec<String>>,
    accepted_sub: Option<String>,
    decode_algorithms: Vec<Algorithm>,
}

impl Default for JwtConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl JwtConfigBuilder {
    pub fn new() -> Self {
        Self {
            encoding: AlgKeyPair::default(),
            decoding: vec![AlgKeyPair::default()],
            validation: ValidationConfig::default(),
            iss_config: IssueConfig::default(),
        }
    }

    pub fn iss_config(&self) -> &IssueConfig {
        &self.iss_config
    }

    pub fn build(self) -> Result<JwtConfig, MultiCliError> {
        let mut errors = MultiCliError::new();

        let decoding_key = self
            .decoding
            .iter()
            .filter_map(|pair| match pair.build_as_decode_key() {
                Ok(alg_key_pair) => Some(alg_key_pair),
                Err(e) => {
                    errors.add(e);
                    None
                }
            })
            .collect();

        let encoding_key = match self.encoding.build_as_encode_key() {
            Ok(alg_key_pair) => alg_key_pair.1,
            Err(e) => {
                errors.add(e);
                return Err(errors);
            }
        };

        if !errors.is_empty() {
            return Err(errors);
        }

        let res = JwtConfig {
            encoding_key,
            decoding_key,
            header: Header::new(self.encoding.algorithm),
            validation: self.validation.into(),
            uuid_generation: Uuid::new_v4,
        };

        if !res.decoding_key.contains_key(&self.encoding.algorithm) {
            tracing::warn!(
                "no decoding key provided for encoding algorithm {:?}; tokens signed by this server might not be verifiable",
                self.encoding.algorithm
            );
        }

        Ok(res)
    }
}

impl IssueConfig {
    pub fn issue_as(&self) -> Option<&str> {
        match &self.issue_as {
            Some(iss) => Some(iss),
            None => None,
        }
    }

    pub fn issue_to(&self) -> Option<&[String]> {
        match &self.issue_to {
            Some(aud) => Some(aud.as_slice()),
            None => None,
        }
    }

    pub fn expire_in(&self) -> Duration {
        match self.expire_in {
            Some(duration) => duration,
            None => Duration::seconds(3600),
        }
    }

    pub fn not_before(&self) -> Duration {
        match self.not_before {
            Some(duration) => duration,
            None => Duration::seconds(0),
        }
    }
}

impl AlgKeyPair {
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
                "the secret key `{}` is too short to prevent brute force cracking",
                self.key
                    .get(0..4)
                    .map(|val| format!("{val}..."))
                    .unwrap_or(self.key.clone())
            )
        }

        Ok(res)
    }

    fn build_as_encode_key(&self) -> CliResult<(Algorithm, EncodingKey)> {
        if self.form.is_der() {
            let build_from_der = match self.algorithm {
                Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => EncodingKey::from_secret,
                Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 => EncodingKey::from_rsa_der,
                Algorithm::PS256 | Algorithm::PS384 | Algorithm::PS512 => EncodingKey::from_rsa_der,
                Algorithm::ES256 | Algorithm::ES384 => EncodingKey::from_ec_der,
                Algorithm::EdDSA => EncodingKey::from_ed_der,
            };

            Ok((
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

    fn build_as_decode_key(&self) -> CliResult<(Algorithm, DecodingKey)> {
        if self.form.is_der() {
            let build_from_der = match self.algorithm {
                Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => DecodingKey::from_secret,
                Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 => DecodingKey::from_rsa_der,
                Algorithm::PS256 | Algorithm::PS384 | Algorithm::PS512 => DecodingKey::from_rsa_der,
                Algorithm::ES256 | Algorithm::ES384 => DecodingKey::from_ec_der,
                Algorithm::EdDSA => DecodingKey::from_ed_der,
            };

            Ok((
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

impl ValidationConfig {
    pub fn new(alg: Algorithm) -> Self {
        let required_claims = vec!["exp".into()];

        Self {
            required_spec_claims: required_claims,
            leeway: 0,
            reject_tokens_expiring_in_less_than: 0,

            validate_exp: true,
            validate_nbf: false,

            accepted_iss: None,
            accepted_aud: None,
            accepted_sub: None,

            decode_algorithms: vec![alg],
        }
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self::new(Algorithm::default())
    }
}

impl From<ValidationConfig> for Validation {
    /// 执行配置文件到 [`Validation`] 的转化
    fn from(value: ValidationConfig) -> Self {
        let ValidationConfig {
            required_spec_claims,
            leeway,
            reject_tokens_expiring_in_less_than,
            validate_exp,
            validate_nbf,
            accepted_aud: aud,
            accepted_iss: iss,
            accepted_sub: sub,
            decode_algorithms: algorithms,
        } = value;

        let mut validation = Validation::default();
        validation.required_spec_claims = required_spec_claims.into_iter().collect();
        validation.leeway = leeway;
        validation.reject_tokens_expiring_in_less_than = reject_tokens_expiring_in_less_than;
        validation.validate_exp = validate_exp;
        validation.validate_nbf = validate_nbf;
        validation.validate_aud = matches!(&aud, Some(val) if !val.is_empty());
        validation.aud = aud.map(|val| val.into_iter().collect());
        validation.iss = iss.map(|val| val.into_iter().collect());
        validation.sub = sub;
        validation.algorithms = algorithms;

        validation
    }
}
