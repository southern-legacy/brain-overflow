use std::borrow::Cow;

use crate::{
    entity::usr::usr_info::UsrInfo,
    http::{
        api::{
            usr::{check_passwd, validate_passwd, UsrIdent}, ApiResult
        },
        extractor::ValidJson,
        jwt::Jwt,
        utils,
    },
    server::ServerState,
};

use axum::{debug_handler, extract::State, http::StatusCode, response::IntoResponse};
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
    ValidJson(param): ValidJson<LoginParam>,
) -> ApiResult {
    let method = &param.method;
    let res = match method {
        LoginMethod::Phone(phone) => UsrInfo::fetch_all_fields_by_phone(state.db(), phone).await,
        LoginMethod::Email(email) => UsrInfo::fetch_all_fields_by_email(state.db(), email).await,
        LoginMethod::Id(id) => UsrInfo::fetch_all_fields_by_id(state.db(), *id).await,
    };

    let res = match res {
        Ok(val) => val,
        Err(e) => {
            if e.is_not_found() {
                return Err(StatusCode::UNAUTHORIZED.into_response())
            } else {
                return Err(e.into_response())
            }
        }
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
