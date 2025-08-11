mod bio;
mod delete_account;
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
        .route(
            "/delete_account",
            routing::post(delete_account::delete_account),
        )
        .route("/bio", routing::get(bio::bio_get))
        .route_layer(&*AUTH_LAYER)
        .route("/info/{id}", routing::get(info::info))
        .route("/login", routing::post(login::login))
        .route("/signup", routing::post(signup::signup))
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
        match UsrInfo::find_by_id(db, self.id).await? {
            Some(usr_info) => Ok(usr_info),
            None => Err(StatusCode::UNAUTHORIZED.into_response()),
        }
    }
}

/// ## 验证密码复杂度
///
/// 三种字符，字母、数字、特殊字符，此函数将统计字母数、数字数、特殊字符数
///
/// 每种字符如果总数大于2，将被统计进字符种类数，如密码 "01234567891a" 就只算**一种字符**，因为只有**一个字母 'a'**
///
/// 这三种字符中必须有**两种以上**，如不满足，则无法通过校验
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

    if alphas + numerics + specials <= 12 {
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
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error occurs while checking your password, which is my fault!",
            )
                .into_response())
        }
    }
}
