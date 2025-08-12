use std::borrow::Cow;

use crate::entity::usr::usr_info::UsrInfo;
use crate::http::api::ApiResult;
use crate::http::api::usr::{UsrIdent, check_passwd, validate_passwd};
use crate::http::jwt::Jwt;
use crate::http::utils;
use crate::server::ServerState;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, debug_handler};
use axum_valid::Valid;
use serde::Deserialize;
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum LoginMethod {
    Id(i64),
    Email(String),
    Phone(String),
}

#[derive(Validate, Deserialize)]
pub(super) struct LoginParam {
    #[serde(flatten)]
    #[validate(nested)]
    method: LoginMethod,

    #[validate(custom(function = "validate_passwd"))]
    passwd: String,
}

#[debug_handler]
#[tracing::instrument(name = "[usr/login]", skip_all, fields(login_method = %param.method.get_anyway()))]
pub(super) async fn login(
    state: State<ServerState>,
    Valid(Json(param)): Valid<Json<LoginParam>>,
) -> ApiResult {
    let method = &param.method;
    let res = match method {
        LoginMethod::Phone(phone) => UsrInfo::fetch_all_fields_by_phone(state.db(), phone).await?,
        LoginMethod::Email(email) => UsrInfo::fetch_all_fields_by_email(state.db(), email).await?,
        LoginMethod::Id(id) => UsrInfo::fetch_all_fields_by_id(state.db(), *id).await?,
    };

    Ok(check_passwd_and_respond(res, &param.passwd).await?)
}

async fn check_passwd_and_respond(usr: UsrInfo, passwd: &str) -> ApiResult {
    check_passwd(&usr, passwd).await?;

    Ok((
        StatusCode::OK,
        Jwt::generate(UsrIdent {
            email: usr.email,
            phone: usr.phone,
            id: usr.id,
            name: usr.name,
        }),
    )
        .into_response())
}

impl LoginMethod {
    fn get_anyway<'a>(&'a self) -> Cow<'a, str> {
        match self {
            LoginMethod::Id(id) => Cow::Owned(id.to_string()),
            LoginMethod::Email(val) | LoginMethod::Phone(val) => Cow::Borrowed(val),
        }
    }
}

impl Validate for LoginMethod {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        match self {
            LoginMethod::Id(_) => Ok(()),
            LoginMethod::Email(email) => {
                if !utils::meet_email_format(email) {
                    let mut errors = ValidationErrors::new();
                    errors.add(
                        "format",
                        ValidationError::new("1").with_message(Cow::Borrowed(
                            "email address didn't meet the reqiurement of format",
                        )),
                    );
                    Err(errors)
                } else {
                    Ok(())
                }
            }
            LoginMethod::Phone(phone) => {
                if !utils::meet_phone_format(phone) {
                    let mut errors = ValidationErrors::new();
                    errors.add(
                        "format",
                        ValidationError::new("1").with_message(Cow::Borrowed(
                            "email number didn't meet the reqiurement of format",
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
