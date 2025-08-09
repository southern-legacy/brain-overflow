mod delete_account;
mod info;
mod login;
mod signup;

use std::borrow::Cow;
use std::sync::LazyLock;

use crate::entity::usr::prelude::UsrInfo;
use crate::entity::usr::usr_info::Model;
use crate::http::{middelware::auth::AUTH_LAYER, utils};
use crate::server::ServerState;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{routing, Extension, Router};
use sea_orm::{DbConn, EntityTrait};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError, ValidationErrors};

const ARGON2_CONFIG: LazyLock<argon2::Config> = LazyLock::new(|| argon2::Config::default());

pub(super) fn build_router() -> Router<ServerState> {
    let router = Router::new();
    router
        .route(
            "/delete_account",
            routing::post(delete_account::delete_account),
        )
        .route("/info", routing::get(info::info))
        .route_layer(&*AUTH_LAYER)
        .route("/login", routing::post(login::login_by_email_or_phone))
        .route("/login/{id}", routing::post(login::login_by_id))
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
    pub async fn retreive_self_from_db(&self, db: &DbConn) -> Result<Model, Response> {
        match UsrInfo::find_by_id(self.id).one(db).await {
            Ok(Some(model)) => Ok(model),
            Ok(None) => Err(StatusCode::UNAUTHORIZED.into_response()),
            Err(_) => {
                tracing::error!("Cannot find the user {}", self.id);
                Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
            },
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
enum LoginMethod {
    Phone(String),
    Email(String),
}

#[derive(Deserialize, Validate)]
#[serde(deny_unknown_fields)]
struct Param {
    #[validate(nested)]
    #[serde(flatten)]
    method: LoginMethod,

    passwd: String,
}

impl LoginMethod {
    fn get_phone(&self) -> Option<&str> {
        use LoginMethod::*;
        match self {
            Phone(phone) => Some(phone),
            Email(_email) => None,
        }
    }

    fn get_email(&self) -> Option<&str> {
        use LoginMethod::*;
        match self {
            Phone(_phone) => None,
            Email(email) => Some(email),
        }
    }

    fn get_anyway(&self) -> &str {
        use LoginMethod::*;
        match self {
            Phone(v) => v,
            Email(v) => v,
        }
    }
}

impl Validate for LoginMethod {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        use LoginMethod::*;
        match self {
            Phone(phone) => {
                if !utils::meet_phone_format(phone) {
                    let mut errors = ValidationErrors::new();
                    errors.add("format", 
                        ValidationError::new("1")
                            .with_message(Cow::Borrowed("phone number didn't meet the reqiurement of format"))
                    );
                    Err(errors)
                } else {
                    Ok(())
                }
            },

            Email(email) => {
                if !utils::meet_email_format(email) {
                    let mut errors = ValidationErrors::new();
                    errors.add("format", 
                    ValidationError::new("1")
                            .with_message(Cow::Borrowed("email number didn't meet the reqiurement of format"))
                    );
                    Err(errors)
                } else {
                    Ok(())
                }
            },
        }
    }
}

async fn danger_zone_auth(
    db: &DbConn,
    ident: Extension<UsrIdent>,
    passwd: String,
) -> Result<Model, Response> {
    let res = UsrInfo::find_by_id(ident.id).one(db).await;
    match res {
        Ok(Some(val)) => {
            tracing::info!("Found the specified account!");
            match argon2::verify_encoded(&val.passwd_hash, passwd.as_bytes()) {
                Ok(true) => {
                    Ok(val)
                },
                Ok(false) => {
                    tracing::info!("User intended to delete the account with incrrect pasword");
                    Err(StatusCode::UNAUTHORIZED.into_response())
                }
                Err(e) => {
                    tracing::error!("Error checking password! {e}");
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Error occurs while checking your account info! Please change your pasword!"
                    ).into_response())
                }
            }
        },
        Ok(None) => {
            tracing::info!("Seems like there's no user in database.");
            Err(StatusCode::UNAUTHORIZED.into_response())
        },
        Err(err) => {
            tracing::error!("Failed to find the user in the database! {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Can't locate your account!").into_response())
        }
    }
}