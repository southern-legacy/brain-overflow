mod bio;
mod danger_zone;
mod info;
mod login;
mod signup;

use crab_vault_auth::{Jwt, JwtConfig};
use std::sync::LazyLock;

use crate::app_config;
use crate::entity::usr::usr_info::UsrInfo;
use crate::http::middleware::auth::AuthLayer;
use crate::server::ServerState;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Router, routing};
use base64::Engine;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

static ARGON2_CONFIG: LazyLock<argon2::Config> = LazyLock::new(argon2::Config::default);

pub(super) fn build_router() -> Router<ServerState> {
    let router = Router::new();
    router
        .route("/", routing::delete(danger_zone::delete_account))
        .route("/", routing::put(danger_zone::change_auth_info))
        .route("/bio", routing::get(bio::bio_get))
        .layer(AuthLayer::new())
        .route("/{id}", routing::get(info::info))
        .route("/", routing::post(signup::signup))
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
    pub async fn retrieve_self_from_db(&self, db: &PgPool) -> Result<UsrInfo, Response> {
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

    pub fn issue_as_jwt(self, config: &JwtConfig) -> Response {
        let iss_config = app_config::server().auth().iss_config();

        (
            StatusCode::OK,
            Jwt::encode(
                &Jwt::new(self)
                    .issue_as_option(iss_config.issue_as())
                    .audiences_option(iss_config.issue_to())
                    .expires_in(iss_config.expire_in())
                    .not_valid_in(iss_config.not_before()),
                config,
            ),
        )
            .into_response()
    }
}

impl From<UsrInfo> for UsrIdent {
    fn from(usr: UsrInfo) -> Self {
        Self {
            email: usr.email,
            phone: usr.phone,
            id: usr.id,
            name: usr.name,
        }
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
                "Authorization of user (id: {}) failed for incorrect password.",
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

#[tracing::instrument(name = "[usr/generate password]", skip_all)]
async fn generate_passwd_hash(passwd: &str) -> Result<String, Response> {
    let salt = BASE64_STANDARD_NO_PAD.encode(uuid::Uuid::new_v4().into_bytes());
    match argon2::hash_encoded(passwd.as_bytes(), salt.as_bytes(), &ARGON2_CONFIG) {
        Ok(val) => Ok(val),
        Err(e) => {
            tracing::error!("Error occurred while hashing the password! Details: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}
