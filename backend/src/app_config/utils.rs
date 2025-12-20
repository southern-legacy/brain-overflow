use std::collections::HashMap;

use base64::{Engine, prelude::BASE64_STANDARD};
use chrono::TimeDelta;
use clap::error::ErrorKind;
use crab_vault::auth::{JwtDecoder, JwtEncoder};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};

use crate::{
    app_config::ConfigItem,
    error::fatal::{FatalError, FatalResult, MultiFatalError},
};

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct StaticJwtEncoderConfig {
    encoding_keys: Vec<Key>,
    issue_as: String,
    audience: Vec<String>,
    expires_in: i64,
    not_valid_in: i64,
}

pub struct JwtEncoderConfig {
    pub encoder: JwtEncoder,
    pub issue_as: String,
    pub audience: Vec<String>,
    pub expires_in: TimeDelta,
    pub not_valid_in: TimeDelta,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct StaticJwtDecoderConfig {
    /// 主键是 issuer，对应的值是 [`KeyInfo`]
    decoding_keys: Vec<(String, Key)>,
    leeway: u64,
    reject_tokens_expiring_in_less_than: u64,
    audience: Vec<String>,
}

pub struct JwtDecoderConfig {
    pub decoder: JwtDecoder,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Key {
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

impl ConfigItem for StaticJwtEncoderConfig {
    type RuntimeConfig = JwtEncoderConfig;

    fn into_runtime(self) -> FatalResult<JwtEncoderConfig> {
        let StaticJwtEncoderConfig {
            encoding_keys,
            issue_as,
            audience,
            expires_in,
            not_valid_in,
        } = self;

        let (mut keys, mut errors, mut algs, mut kids) =
            (HashMap::new(), MultiFatalError::new(), vec![], vec![]);

        for key in encoding_keys {
            match key.build_as_encode_key() {
                Ok((kid, alg, key)) => {
                    keys.insert(kid.clone(), (key, alg));
                    algs.push(alg);
                    kids.push(kid);
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        if keys.is_empty() {
            errors.push(FatalError::new(
                ErrorKind::Io,
                "you should feed me at least one kid, encoding key pair".to_string(),
                None,
            ));
        }

        if errors.is_empty() {
            Ok(JwtEncoderConfig {
                encoder: JwtEncoder::new(keys),
                issue_as,
                audience,
                expires_in: TimeDelta::new(expires_in, 0).unwrap(),
                not_valid_in: TimeDelta::new(not_valid_in, 0).unwrap(),
            })
        } else {
            Err(errors)
        }
    }
}

impl ConfigItem for StaticJwtDecoderConfig {
    type RuntimeConfig = JwtDecoderConfig;

    fn into_runtime(self) -> FatalResult<JwtDecoderConfig> {
        let StaticJwtDecoderConfig {
            decoding_keys,
            leeway,
            reject_tokens_expiring_in_less_than,
            audience: aud,
        } = self;
        let (mut keys, mut errors, mut algs, mut issuers) =
            (HashMap::new(), MultiFatalError::new(), vec![], vec![]);

        for (iss, key) in decoding_keys {
            match key.build_as_decode_key() {
                Ok((kid, alg, key)) => {
                    issuers.push(iss.clone());
                    algs.push(alg);
                    keys.insert((iss, kid), key);
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        if keys.is_empty() {
            errors.push(FatalError::new(
                ErrorKind::Io,
                "you should feed me at least one kid, encoding key pair".to_string(),
                None,
            ));
        }

        if errors.is_empty() {
            Ok(JwtDecoderConfig {
                decoder: JwtDecoder::new(keys, &algs, &issuers, &aud)
                    .reject_tokens_expiring_in_less_than(reject_tokens_expiring_in_less_than)
                    .leeway(leeway),
            })
        } else {
            Err(errors)
        }
    }
}

impl Key {
    fn get_key(&self) -> Result<Vec<u8>, FatalError> {
        let res = match self.form {
            KeyForm::DerInline => BASE64_STANDARD.decode(self.key.clone()).map_err(|e| {
                FatalError::from(e).when(format!(
                    "while decoding the secrete key `{}` into binary, note this should be encoded in standard base64",
                    self.key
                        .get(0..4)
                        .map(|val| format!("{val}..."))
                        .unwrap_or(self.key.clone())
                ))
            })?,
            KeyForm::DerFile => std::fs::read(&self.key).map_err(|e| {
                FatalError::from(e).when(format!("while reading the der key from {}", self.key))
            })?,
            KeyForm::PemInline => self.key.clone().into_bytes(),
            KeyForm::PemFile => std::fs::read(&self.key).map_err(|e| {
                FatalError::from(e).when(format!("while reading the pem key from {}", self.key))
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

    fn build_as_encode_key(&self) -> Result<(String, Algorithm, EncodingKey), FatalError> {
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
                    e.when("while building jwt encoding key from a der form".into())
                })?),
            ))
        } else if self.form.is_pem() {
            let build_from_pem = match self.algorithm {
                Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                    return Err(FatalError::new(
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
                    e.when("while building jwt encoding key from a pem form".into())
                })?)
                .map_err(|e| {
                    FatalError::new(
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

    fn build_as_decode_key(&self) -> Result<(String, Algorithm, DecodingKey), FatalError> {
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
                    e.when("while building jwt encoding key from a der form".into())
                })?),
            ))
        } else if self.form.is_pem() {
            let build_from_pem = match self.algorithm {
                Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                    return Err(FatalError::new(
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
                    FatalError::new(
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
    #[inline]
    fn is_der(&self) -> bool {
        matches!(self, KeyForm::DerInline | KeyForm::DerFile)
    }

    #[inline]
    fn is_pem(&self) -> bool {
        matches!(self, KeyForm::PemInline | KeyForm::PemFile)
    }
}
