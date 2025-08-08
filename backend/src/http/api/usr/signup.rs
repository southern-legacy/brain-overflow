use std::borrow::Cow;

use axum::{debug_handler, extract::State, http::StatusCode, response::IntoResponse, Json};
use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine};
use sea_orm::{ActiveValue::{NotSet, Set}, EntityTrait, IntoActiveModel};
use serde::Deserialize;
use validator::{Validate, ValidationError, ValidationErrors};

use crate::entity::usr::{prelude::UsrInfo, usr_info::{self}};
use crate::http::{
    api::usr::ARGON2_CONFIG,
    jwt::Jwt,
    middelware::auth::UsrIdent,
    utils
};
use crate::server::ServerState;

#[derive(Deserialize)]
pub(super) struct SignupParam {
    name: String,
    email: Option<String>,
    phone: Option<String>,
    passwd: String,
}

impl Validate for SignupParam {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        if self.name.len() > 32 {
            errors.add(
                "Name",
                ValidationError::new("Too long for a name").with_message(Cow::Borrowed("Expected ONE sign up method"))
            );
        }

        if !(self.email.is_some() ^ self.phone.is_some()) {
            errors.add(
                "Sign up method",
                ValidationError::new("Incorrect method").with_message(Cow::Borrowed("Expected ONE sign up method"))
            );
        }

        if self.email.is_some() && !utils::meet_email_format(&self.email.as_ref().unwrap()) {
            errors.add(
                "Validation",
                ValidationError::new("format invalid").with_message(Cow::Borrowed("email address format is invalid")),
            );
        }

        if self.phone.is_some() && !utils::meet_phone_format(&self.phone.as_ref().unwrap()) {
            errors.add(
                "Validation",
                ValidationError::new("format invalid").with_message(Cow::Borrowed("phone number format is invalid")),
            );
        }

        if errors.is_empty() { Ok(()) }
        else { Err(errors) }
    }
}

#[debug_handler]
pub(super) async fn signup(
    state: State<ServerState>,
    param: Json<SignupParam>
) -> impl IntoResponse {
    let salt = BASE64_STANDARD_NO_PAD.encode(uuid::Uuid::new_v4().into_bytes());
    let SignupParam { name, email, phone, passwd } = param.0;

    let new_usr = usr_info::ActiveModel {
        id: NotSet,
        name: Set(name.clone()),
        email: Set(email.clone()),
        phone: Set(phone.clone()),
        salt: Set(salt.clone()),
        passwd_hash: match argon2::hash_encoded(passwd.as_bytes(), salt.as_bytes(), &ARGON2_CONFIG) {
            Ok(val) => Set(val),
            Err(e) => {
                tracing::error!("Error occured while hashing the password! {e}");
                return (StatusCode::INTERNAL_SERVER_ERROR, Cow::Borrowed(""))
            }
        }
    };

    match UsrInfo::insert(new_usr.into_active_model()).exec(state.db()).await {
        Ok(val) => {
            tracing::info!("Successfully insert a user into database.");
            (StatusCode::OK, Cow::Owned(Jwt::generate(UsrIdent {
                id: val.last_insert_id,
                email, phone, name,
            })))
        },
        Err(e) => {
            tracing::error!("Failed to sign up a user! details: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Cow::Owned("Failed to sign up".to_string()))
        },
    }
}