mod bio;
mod delete;
mod info;
mod login;
mod signup;

use std::borrow::Cow;
use std::sync::LazyLock;

use crate::entity::usr::usr_info::UsrInfo;
use crate::http::middelware::auth::AUTH_LAYER;
use crate::server::ServerState;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Router, routing};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use validator::ValidationError;

const ARGON2_CONFIG: LazyLock<argon2::Config> = LazyLock::new(|| argon2::Config::default());

pub(super) fn build_router() -> Router<ServerState> {
    let router = Router::new();
    router
        /* 删除用户 */
        .route("/", routing::delete(delete::delete_account))
        /* 用户自视 */
        .route("/bio", routing::get(bio::bio_get))
        /* 必须验证 */
        .route_layer(&*AUTH_LAYER)
        /* 读取用户 */
        .route("/{id}", routing::get(info::info))
        /* 创建用户 */
        .route("/", routing::post(signup::signup))
        /* 创建会话 */
        .route("/login", routing::post(login::login))
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UsrIdent {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}

impl UsrIdent {
    pub async fn retreive_self_from_db(&self, db: &PgPool) -> Result<UsrInfo, Response> {
        match UsrInfo::fetch_all_fields_by_id(db, self.id).await {
            Ok(usr_info) => Ok(usr_info),
            Err(e) => {
                if e.is_not_found() {
                    Err(StatusCode::UNAUTHORIZED.into_response())
                } else {
                    Err(Response::from(e))
                }
            }
        }
    }
}

/// ## 验证密码复杂度
///
/// 三种字符，字母、数字、特殊字符，此函数将统计字母数、数字数、特殊字符数
///
/// 每种字符如果总数大于2，将被统计进字符种类数，如密码 "01234567891a" 就只算**一种字符**，因为只有**一个字母 'a'**
///
/// 通过校验需要满足两个条件：
///
/// - 这三种字符中必须有**两种以上**
/// - 密码整体长度大于等于 12
///
/// 同时注意：密码使用 Unicode 字符集，所以基本所有的字符都能作为密码的一部分
///
fn validate_passwd(val: &str) -> Result<(), ValidationError> {
    let mut alphas = 0;
    let mut numerics = 0;
    let mut specials = 0;
    for c in val.chars() {
        if c.is_alphabetic() {
            alphas += 1;
        } else if c.is_numeric() {
            numerics += 1;
        } else {
            specials += 1;
        }
    }

    let count = |val| match val {
        2.. => 1,
        _ => 0,
    };

    let count = count(alphas) + count(numerics) + count(specials);

    if alphas + numerics + specials < 12 {
        Err(ValidationError::new("password").with_message(Cow::Borrowed("Password is too short!")))
    } else if count < 2 {
        Err(ValidationError::new("password").with_message(Cow::Borrowed("Password is too simple!")))
    } else {
        Ok(())
    }
}

#[tracing::instrument(name = "[usr/check password]", skip_all)]
async fn check_passwd(val: &UsrInfo, passwd: &str) -> Result<(), Response> {
    match argon2::verify_encoded(&val.passwd_hash, passwd.as_bytes()) {
        Ok(true) => {
            tracing::info!("Authorization of user (id: {}) successfully.", val.id);
            Ok(())
        }
        Ok(false) => {
            tracing::info!(
                "Authorization of user (id: {}) failed for incorrect pasword.",
                val.id
            );
            Err(StatusCode::UNAUTHORIZED.into_response())
        }
        Err(e) => {
            tracing::error!("Error checking password! {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}
