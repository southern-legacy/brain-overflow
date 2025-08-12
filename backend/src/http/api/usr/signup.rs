use std::borrow::Cow;

use axum::{
    Json, debug_handler,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
};
use axum_valid::Valid;
use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use serde::Deserialize;
use validator::{Validate, ValidationError, ValidationErrors};

use crate::server::ServerState;
use crate::{
    entity::usr::usr_info::{InsertParam, UsrInfo},
    http::{
        api::{
            ApiResult,
            usr::{ARGON2_CONFIG, UsrIdent, validate_passwd},
        },
        utils,
        jwt::Jwt,
    },
};

#[derive(Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
enum SignUpMethod {
    Phone(String),
    Email(String),
}

type Email = Option<String>;
type Phone = Option<String>;
impl SignUpMethod {
    fn get_tup_phone_email(self) -> (Phone, Email) {
        use SignUpMethod::*;
        match self {
            Phone(phone) => (Some(phone), None),
            Email(email) => (None, Some(email)),
        }
    }

    fn get_anyway(&self) -> &str {
        use SignUpMethod::*;
        match self {
            Phone(v) => v,
            Email(v) => v,
        }
    }
}

#[derive(Deserialize, Validate)]
pub(super) struct SignUpParam {
    #[validate(length(max = 32))]
    name: String,

    #[validate(nested)]
    #[serde(flatten)]
    method: SignUpMethod,

    #[validate(custom(function = "validate_passwd"))]
    passwd: String,
}

#[debug_handler]
#[tracing::instrument(name = "[usr/signup]", skip_all, fields(verification = %param.method.get_anyway()))]
pub(super) async fn signup(
    state: State<ServerState>,
    Valid(Json(param)): Valid<Json<SignUpParam>>,
) -> ApiResult {
    let salt = BASE64_STANDARD_NO_PAD.encode(uuid::Uuid::new_v4().into_bytes());
    let SignUpParam { name, method, passwd } = param;

    let (phone, email) = method.get_tup_phone_email();

    let new_usr = InsertParam {
        email: email.clone(),
        phone: phone.clone(),
        name: name.clone(),
        passwd: match argon2::hash_encoded(passwd.as_bytes(), salt.as_bytes(), &ARGON2_CONFIG) {
            Ok(val) => val,
            Err(e) => {
                tracing::error!("Error occurred while hashing the password! {e}");
                return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
            }
        },
    };

    let id = UsrInfo::insert_and_return_id(state.db(), new_usr).await?;
    tracing::info!("Successfully inserted a user into database.");
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, format!("/usr/{}", id))],
        Jwt::generate(UsrIdent {
            id, name, email, phone
        }),
    )
        .into_response())
}

impl Validate for SignUpMethod {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        use SignUpMethod::*;
        match self {
            Phone(phone) => {
                if !utils::meet_phone_format(phone) {
                    let mut errors = ValidationErrors::new();
                    errors.add(
                        "format",
                        ValidationError::new("1").with_message(Cow::Borrowed(
                            "phone number didn't meet the requirement of format",
                        )),
                    );
                    Err(errors)
                } else {
                    Ok(())
                }
            }

            Email(email) => {
                if !utils::meet_email_format(email) {
                    let mut errors = ValidationErrors::new();
                    errors.add(
                        "format",
                        ValidationError::new("1").with_message(Cow::Borrowed(
                            "email number didn't meet the requirement of format",
                        )),
                    );
                    Err(errors)
                } else {
                    Ok(())
                }
            }
        }
    }
}