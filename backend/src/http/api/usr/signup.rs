use axum::{
    debug_handler,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
};
use serde::Deserialize;
use validator::{Validate, ValidationErrors};

use crate::{
    entity::usr::usr_info::{InsertParam, UsrInfo},
    http::{
        api::{
            ApiResult,
            usr::{UsrIdent, generate_passwd_hash},
        },
        extractor::ValidJson,
        jwt::Jwt,
        utils::{validate_email, validate_passwd, validate_phone},
    },
    server::ServerState,
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
#[tracing::instrument(name = "[usr/signup]", skip_all, fields(verify = %param.method.get_anyway()))]
pub(super) async fn signup(
    State(state): State<ServerState>,
    ValidJson(param): ValidJson<SignUpParam>,
) -> ApiResult {
    let SignUpParam {
        name,
        method,
        passwd,
    } = param;

    let (phone, email) = method.get_tup_phone_email();

    let passwd_hash = generate_passwd_hash(&passwd).await?;

    let new_usr = InsertParam {
        email: email.as_ref(),
        phone: phone.as_ref(),
        name: &name,
        passwd: &passwd_hash,
    };

    let id = UsrInfo::insert_and_return_id(state.db(), new_usr).await?;
    tracing::info!("Successfully inserted a user into database.");
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, format!("/usr/{}", id))],
        Jwt::generate(UsrIdent {
            id,
            name,
            email,
            phone,
        }),
    )
        .into_response())
}

impl Validate for SignUpMethod {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        use SignUpMethod::*;
        let mut errors = ValidationErrors::new();
        match self {
            Email(email) => match validate_email(email) {
                Ok(_) => Ok(()),
                Err(e) => {
                    errors.add("email", e);
                    Err(errors)
                }
            },
            Phone(phone) => match validate_phone(phone) {
                Ok(_) => Ok(()),
                Err(e) => {
                    errors.add("phone", e);
                    Err(errors)
                }
            },
        }
    }
}
