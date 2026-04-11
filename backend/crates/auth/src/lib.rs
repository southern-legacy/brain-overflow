pub mod error;

use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use base64::Engine;
use jsonwebtoken::{DecodingKey, Validation};

use crate::error::AuthError;

#[derive(Clone)]
pub struct JwtEncoder {
    /// 用于签发 JWT 的密钥。从 kid 到 ([`EncodingKey`], [`Algorithm`]) 的映射
    pub encoding_key: HashMap<String, (EncodingKey, Algorithm)>,

    kids: Vec<String>,
}

#[derive(Clone)]
pub struct JwtDecoder {
    /// 用于验证 JWT 的密钥映射。
    ///
    /// [`HashMap`] 的键是签发者 (iss, kid)，值是对应的轮换密钥 ([`DecodingKey`])。
    decoding_keys: HashMap<(String, String), DecodingKey>,

    /// JWT 的验证规则。
    ///
    /// 用于配置如何验证 `exp`, `nbf`, `iss`, `aud` 等标准声明。
    validation: Validation,
}

/// ## 表示一个完整的 JWT，包含标准声明和自定义载荷。
///
/// 泛型参数 `P` 代表自定义的载荷 (Payload) 结构体。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Jwt<P> {
    /// (Issuer) 签发者
    pub iss: String,

    /// (Audience) 受众。可以是一个或多个。
    pub aud: Vec<String>,

    /// (Expiration Time) 过期时间。Unix 时间戳。
    pub exp: i64,

    /// (Not Before) 生效时间。Unix 时间戳。
    pub nbf: i64,

    /// (Issued At) 签发时间。Unix 时间戳。
    pub iat: i64,

    /// (JWT ID) 令牌唯一标识。
    pub jti: Uuid,

    /// 自定义的载荷数据。
    pub load: P,
}

impl JwtEncoder {
    #[inline]
    pub fn new(encoding_key: HashMap<String, (EncodingKey, Algorithm)>) -> Self {
        let kids = encoding_key.keys().cloned().collect();
        Self { encoding_key, kids }
    }

    /// ## 将 JWT 声明编码为字符串形式的 Token
    ///
    /// **注意**：header 中的 alg 字段和 kid 对应的加密算法需要保持一致
    #[inline]
    pub fn encode<P: Serialize>(&self, claims: &Jwt<P>, kid: &str) -> Result<String, AuthError> {
        use AuthError::InternalError;

        let (key, alg) = self
            .encoding_key
            .get(kid)
            .ok_or(InternalError("No such kid found in your encoder".into()))?;

        let mut header = Header::new(*alg);
        header.kid = Some(kid.to_string());

        Ok(jsonwebtoken::encode(&header, claims, key)?)
    }

    pub fn encode_randomly<P: Serialize>(&self, claims: &Jwt<P>) -> Result<String, AuthError> {
        let random_kid = &self.kids[rand::random_range(..self.kids.len())];
        self.encode(claims, random_kid)
    }
}

impl JwtDecoder {
    /// ## 新建一个 [`JwtDecoder`]
    ///
    /// ### 参数说明
    ///
    /// - `mapping` `iss`、`kid` 到 [`DecodingKey`] 的映射，注意  [`mapping`](HashMap) 的联合主键的顺序是 (iss, kid)，别搞反了！
    /// - `algorithms`    接受的算法
    /// - `iss`     接受的令牌的签发人
    /// - `aud`     接受的令牌中的 aud 值
    ///
    /// ### panic
    ///
    /// - 如果 `algorithms` 中一个算法都没有，即 `algorithms` 是一个空的切片
    ///
    /// ### 新建完成后可以通过以下函数修改相应的配置
    ///
    /// - [`iss_kid_dec`](JwtDecoder::iss_kid_dec)
    /// - [`algorithms`](JwtDecoder::algorithms)
    /// - [`authorized_issuer`](JwtDecoder::authorized_issuer)
    /// - [`possible_audience`](JwtDecoder::possible_audience)
    /// - [`leeway`](JwtDecoder::leeway)
    /// - [`reject_tokens_expiring_in_less_than`](JwtDecoder::reject_tokens_expiring_in_less_than)
    ///
    /// ### 然后可以使用方法 [`decode`](JwtDecoder::decode) 来解码、校验一个 jwt
    ///
    pub fn new<T: ToString, U: ToString>(
        mapping: HashMap<(String, String), DecodingKey>,
        algorithms: &[Algorithm],
        iss: &[T],
        aud: &[U],
    ) -> Self {
        let mut validation = Validation::new(
            *algorithms
                .first()
                .expect("You should provide at least one algorithm in your accepted algorithm slice!"),
        );
        validation.validate_aud = true;
        validation.validate_exp = true;
        validation.validate_nbf = true;
        validation.algorithms = algorithms.to_vec();
        validation.reject_tokens_expiring_in_less_than = 0;
        validation.leeway = 60;
        validation.set_issuer(iss);
        validation.set_audience(aud);

        // 必须有下面的四个字段，否则视为非法 token，
        // jsonwebtoken 只接受下面的这些和 sub 字段，所以 iat 限制无法设置
        // 当然，如果没有，serde 也会自己产生反序列化错误，所以应该没问题……吧

        validation.set_required_spec_claims(&["aud", "exp", "nbf", "iss"]);
        Self {
            decoding_keys: mapping,
            validation,
        }
    }

    /// ## 设置 (iss, kid) 到 [`DecodingKey`] 的映射
    ///
    /// 注意  [`mapping`](HashMap) 的联合主键的顺序是 (iss, kid)，别搞反了！
    #[inline]
    pub fn iss_kid_dec(mut self, mapping: HashMap<(String, String), DecodingKey>) -> Self {
        self.decoding_keys = mapping;
        self
    }

    /// ## 设置接受的算法
    #[inline]
    pub fn algorithms(mut self, algorithms: &[Algorithm]) -> Self {
        self.validation.algorithms = algorithms.to_vec();
        self
    }

    /// ## 设置接受的 issuer
    #[inline]
    pub fn authorized_issuer<T: ToString>(mut self, iss: &[T]) -> Self {
        self.validation.set_issuer(iss);
        self
    }

    /// ## 设置接受的 audience
    #[inline]
    pub fn possible_audience<T: ToString>(mut self, aud: &[T]) -> Self {
        self.validation.set_audience(aud);
        self
    }

    /// ## 设置接受的 leeway
    #[inline]
    pub const fn leeway(mut self, leeway: u64) -> Self {
        self.validation.leeway = leeway;
        self
    }

    /// ## 临期的 token 不予通过
    #[inline]
    pub const fn reject_tokens_expiring_in_less_than(mut self, tolerance: u64) -> Self {
        self.validation.reject_tokens_expiring_in_less_than = tolerance;
        self
    }

    /// ## 使用给定的配置解码并验证一个字符串形式的 Token。
    ///
    /// 此函数会执行完整的验证流程，包括：
    /// 1. 检查签名是否有效。
    /// 2. 验证 `exp` 和 `nbf` 时间戳。
    /// 3. 根据 `config.validation` 中的设置验证 `iss` 和 `aud`。
    ///
    /// ### 泛型参数说明
    ///
    /// 注意这个函数的泛型参数 `P` 代表的是 **载荷 (Payload)** 的类型，而不是 `Jwt` 本身。
    ///
    /// ### 代码示例
    ///
    /// #### 推荐写法 (Best Practice)
    ///
    /// 利用 Rust 的类型推断，显式标注变量类型，代码最为清晰：
    ///
    /// ```rust,no_run
    /// # use ::auth::{JwtDecoder, Jwt, error::AuthError};
    /// # fn example(decoder: &JwtDecoder, token: &str) -> Result<(), AuthError> {
    /// // 编译器会自动推断出 P 是 ()
    /// let jwt: Jwt<()> = decoder.decode(token)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// #### 显式泛型写法
    ///
    /// 也可以使用 Turbofish 语法显式指定载荷类型：
    ///
    /// ```rust,no_run
    /// # use ::auth::{JwtDecoder, Jwt, error::AuthError};
    /// # fn example(decoder: &JwtDecoder, token: &str) -> Result<(), AuthError> {
    /// // 注意：尖括号内只需填 ()
    /// let jwt = decoder.decode::<()>(token)?;
    /// // 此时 jwt 的类型为 Jwt<()>
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// #### 错误写法 (编译失败)
    ///
    /// 不要将 `Jwt<()>` 作为泛型参数传入，否则会导致类型嵌套 (`Jwt<Jwt<P>>`)，
    /// 这会导致类型不匹配从而**编译失败**：
    ///
    /// ```rust,compile_fail
    /// # use ::auth::{JwtDecoder, Jwt, AuthError};
    /// # fn example(decoder: &JwtDecoder, token: &str) -> Result<(), AuthError> {
    /// // 错误：decode 返回的是 Jwt<T>。
    /// // 如果传入 T = Jwt<()>，返回值就是 Jwt<Jwt<()>>。
    /// // 这与左侧的变量类型 Jwt<()> 不匹配。
    /// let jwt: Jwt<()> = decoder.decode::<Jwt<()>>(token)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn decode<P>(&self, token: &str) -> Result<Jwt<P>, AuthError>
    where
        for<'de> P: Deserialize<'de>,
    {
        let kid = jsonwebtoken::decode_header(token)?
            .kid
            .ok_or(AuthError::MissingClaim("kid".to_string()))?;

        let body_unchecked: Jwt<P> = serde_json::from_value(Self::decode_unchecked(token)?)?;

        let key = self
            .decoding_keys
            .get(&(body_unchecked.iss, kid))
            .ok_or(AuthError::InvalidIssuer)?;

        Ok(jsonwebtoken::decode::<Jwt<P>>(token, key, &self.validation)?.claims)
    }

    /// ## **\[不安全\]** 在不验证签名的情况下解码 JWT 的载荷。
    ///
    /// # 警告
    ///
    /// **绝对不要**相信此函数返回的数据！因为它**没有验证** JWT 的签名。
    /// 这意味着任何人都可以伪造这个 JWT 的内容。
    ///
    /// 此函数仅应用于需要查看 Token 内容的调试或日志记录场景。
    /// 在任何与安全相关的逻辑中，都**必须**使用 [`JwtDecoder::decode`]。
    pub fn decode_unchecked(token: &str) -> Result<serde_json::Value, AuthError> {
        let mut parts = token.split('.');
        let _header = parts.next();
        let payload = parts.next().ok_or(AuthError::InvalidToken)?;

        let decoded_payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(payload)?;
        let json_value = serde_json::from_slice(&decoded_payload)?;

        Ok(json_value)
    }
}

impl<P: Serialize + for<'de> Deserialize<'de>> Jwt<P> {
    /// 创建一个新的 `Jwt` 实例，并填入默认值。
    ///
    /// 默认值:
    /// - `iss`: `None`
    /// - `aud`: 空 `Vec`
    /// - `exp`: `一小时后` 的时间戳
    /// - `nbf`: `0` (立即生效)
    /// - `iat`: 当前时间的 Unix 时间戳
    /// - `jti`: 一个使用 [`Uuid::new_v4`] 新生成的 [`Uuid`]
    #[inline]
    pub fn new<T: ToString, U: ToString>(iss: T, aud: &[U], payload: P) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            iss: iss.to_string(),
            aud: aud.iter().map(|s| s.to_string()).collect(),
            exp: now + 3600,
            nbf: now,
            iat: now,
            jti: Uuid::new_v4(),
            load: payload,
        }
    }

    /// 设置 JWT 的相对过期时间，从现在开始计算。
    #[inline]
    pub fn expires_in(mut self, duration: chrono::Duration) -> Self {
        self.exp = (chrono::Utc::now() + duration).timestamp();
        self
    }

    /// 设置 JWT 的过期时间为一个绝对的时间点。
    #[inline]
    pub fn expires_at<T>(mut self, when: chrono::DateTime<T>) -> Self
    where
        T: chrono::TimeZone,
    {
        self.exp = when.timestamp();
        self
    }

    /// !!! 永不过期 !!!
    #[inline]
    pub const fn never_expires(mut self) -> Self {
        self.exp = i32::MAX as i64;
        self
    }

    /// 设置 JWT 的生效时间，从现在开始计算。
    #[inline]
    pub fn not_valid_in(mut self, duration: chrono::Duration) -> Self {
        self.nbf = (chrono::Utc::now() + duration).timestamp();
        self
    }

    /// 设置 JWT 的生效时间为一个绝对的时间点。
    #[inline]
    pub fn not_valid_till<T>(mut self, when: chrono::DateTime<T>) -> Self
    where
        T: chrono::TimeZone,
    {
        self.nbf = when.timestamp();
        self
    }

    /// 在构建 token 的时候更换 uuid
    #[inline]
    pub const fn uuid(mut self, id: Uuid) -> Self {
        self.jti = id;
        self
    }
}
