use std::borrow::Cow;

use crate::entity::usr::{prelude::UsrInfo, usr_info};
use crate::http::{jwt::Jwt, middelware::auth::UsrIdent, utils};
use crate::server::ServerState;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, debug_handler};
use axum_valid::Valid;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryTrait};
use serde::Deserialize;
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Deserialize, Debug)]
pub(super) struct LoginParam {
    id: Option<i64>,
    email: Option<String>,
    phone: Option<String>,
    passwd: String,
}

impl Validate for LoginParam {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        let mut count: u8 = 0;
        if self.id.is_some() { count += 1; }
        if self.email.is_some() { count += 1; }
        if self.phone.is_some() { count += 1; }

        if count > 1 {
            errors.add(
                "Login method",
                ValidationError::new("identifier").with_message(Cow::Borrowed("multiple identifier supplied")),
            );
        } else if count == 0 {
            errors.add(
                "Login method",
                ValidationError::new("identifier").with_message(Cow::Borrowed("no identifier supplied")),
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
#[tracing::instrument(name = "[Login]", skip(state))]
pub(super) async fn login(state: State<ServerState>, param: Valid<Json<LoginParam>>) -> impl IntoResponse {
    let res = UsrInfo::find()
        .apply_if(param.id.as_ref(),    |rows, id| rows.filter(usr_info::Column::Id.eq(*id)))
        .apply_if(param.email.as_ref(), |rows, email| rows.filter(usr_info::Column::Email.eq(email)))
        .apply_if(param.phone.as_ref(), |rows, phone| rows.filter(usr_info::Column::Phone.eq(phone)))
        .one(state.db());

    match res.await {
        Ok(Some(val)) => {
            match argon2::verify_encoded(&val.passwd_hash, param.passwd.as_bytes()) {
                Ok(true) => {
                    tracing::info!("User {} login successfully", val.id);
                    (StatusCode::OK, Cow::Owned(Jwt::generate(UsrIdent {
                        id: val.id,
                        name: val.name,
                        email: val.email,
                        phone: val.phone,
                    })))
                },
                Ok(false) => {
                    tracing::info!("User {} login with incrrect pasword", val.id);
                    (StatusCode::UNAUTHORIZED, Cow::Borrowed("Your password or account don't match"))
                }
                Err(e) => {
                    tracing::error!("Error checking password! {e}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Cow::Borrowed("Error occurs while checking your account info! Please change your pasword!")
                    )
                }
            }
        },
        Ok(None) => {
            tracing::info!("No account for this user");
            (StatusCode::UNAUTHORIZED, Cow::Borrowed("You don't have an account!"))
        },
        Err(e) => {
            tracing::error!("Database error! details: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Cow::Owned(e.to_string()))
        }
    }
}