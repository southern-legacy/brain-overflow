use std::{
    sync::LazyLock,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use base64::{Engine, prelude::BASE64_STANDARD};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::app_config;

#[derive(Serialize, Deserialize, Default)]
pub struct JwtClaims {
    exp: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Jwt<T> {
    #[serde(flatten)]
    load: T,

    #[serde(flatten)]
    claims: JwtClaims,
}

static ENCODING_KEY: LazyLock<EncodingKey> = LazyLock::new(|| {
    let key = BASE64_STANDARD
        .decode(app_config::get_server().secret_key())
        .expect("secret_key should be a string encoded in base64.");
    EncodingKey::from_secret(&key)
});

static DECODING_KEY: LazyLock<DecodingKey> = LazyLock::new(|| {
    let key = BASE64_STANDARD
        .decode(app_config::get_server().secret_key())
        .expect("secret_key should be a string encoded in base64.");    
    DecodingKey::from_secret(&key)
});

pub static DEFAULT_VALIDATION: LazyLock<Validation> = LazyLock::new(Validation::default);
pub static DEFAULT_EXPIRATION: LazyLock<Duration> =
    LazyLock::new(|| Duration::from_secs(12 * 60 * 60));

impl<T: Serialize + for<'de> Deserialize<'de>> Jwt<T> {
    /// 快捷方式，过期时间为 12 小时，加密算法为 HS256
    pub fn generate(load: T) -> String {
        Self::new(load, &DEFAULT_EXPIRATION).encode_with(Algorithm::HS256)
    }

    /// 创建一个 jwt, 时间将被转化为秒级时间戳
    /// - `load: T`: 负载
    /// - `ttl: &Duration`: 此 jwt 的生存时长, 默认为 12 小时
    pub fn new(load: T, ttl: &Duration) -> Self {
        Self {
            load,
            claims: JwtClaims {
                exp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    + ttl.as_secs(),
            },
        }
    }

    /// 由于使用 base 64 解码后的东西是一个二进制序列, 所以使用 HMAC 算法组 (哈希算法组)
    /// `alg` 参数仅接受 HMAC 算法组的算法
    pub fn encode_with(self, alg: Algorithm) -> String {
        let header = Header::new(alg);
        // 查看源码可知, encode 函数仅在 header.alg.algorithmfamily 和 key 的 algorithm family不一样的时候
        // 此处: Encoding Key 和 Decoding Key 均为二进制序列, 不可能抛出异常, 故直接 unwrap
        let res = jsonwebtoken::encode(&header, &self, &ENCODING_KEY).unwrap();
        tracing::info!("Generated a JSON Web Token");
        res
    }

    /// 通过 token 解码出 load，val 参数为校验配置，见 [`jsonwebtoken::Validation`]
    /// 默认的 val 参数可以传递 [`crate::route::jwt`] 模块中的 常量 [`crate::route::jwt::DEFAULT_VALIDATION`]
    pub fn decode_with(token: &str, val: &Validation) -> Result<T, jsonwebtoken::errors::Error> {
        let res = jsonwebtoken::decode::<Self>(token, &DECODING_KEY, val);
        match res {
            Ok(res) => Ok(res.claims.load),
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
    struct TestLoad {
        name: String,
        id: u32,
    }

    impl Default for TestLoad {
        fn default() -> Self {
            Self {
                name: "test".to_string(),
                id: 42,
            }
        }
    }

    #[test]
    fn test_jwt_round_trip() {
        let load = TestLoad::default();
        let jwt = Jwt::new(load.clone(), &Duration::from_secs(60));
        let token = jwt.encode_with(Algorithm::HS256);
        let decoded =
            Jwt::<TestLoad>::decode_with(&token, &Validation::new(Algorithm::HS256)).unwrap();
        println!("{token}");
        assert_eq!(load, decoded);
    }

    #[test]
    fn test_jwt_expired() {
        let load = TestLoad::default();
        let jwt = Jwt::new(load.clone(), &Duration::from_secs(1));
        let token = jwt.encode_with(Algorithm::HS256);
        let decoded = Jwt::<TestLoad>::decode_with(&token, &DEFAULT_VALIDATION);
        println!("{token}");
        assert!(!decoded.is_err());
        assert_eq!(load, decoded.unwrap());

        std::thread::sleep(Duration::from_secs(2));

        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.leeway = 0;

        let decoded = Jwt::<TestLoad>::decode_with(&token, &validation);
        assert!(decoded.is_err());
        println!("{}", decoded.unwrap_err());
    }
}