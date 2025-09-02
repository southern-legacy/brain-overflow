use std::borrow::Cow;

use crate::{
    entity::usr::usr_info::UsrInfo,
    http::{
        api::{
            ApiResult,
            usr::{UsrIdent, check_passwd},
        },
        extractor::ValidJson,
        utils::{validate_email, validate_passwd, validate_phone},
    },
    server::ServerState,
};

use axum::{debug_handler, extract::State, http::StatusCode, response::IntoResponse};
use crab_vault_auth::JwtConfig;
use serde::Deserialize;
use validator::{Validate, ValidationErrors};

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
    State(state): State<ServerState>,
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
                return Err(StatusCode::UNAUTHORIZED.into_response());
            } else {
                return Err(e.into_response());
            }
        }
    };

    check_passwd_and_respond(res, &param.passwd, state.jwt_config()).await
}

async fn check_passwd_and_respond(usr: UsrInfo, passwd: &str, config: &JwtConfig) -> ApiResult {
    check_passwd(&usr, passwd).await?;

    Ok(UsrIdent::from(usr).issue_as_jwt(config))
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
        use LoginMethod::*;
        if let Id(_) = self {
            return Ok(());
        }
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
            Id(_) => unreachable!(),
        }
    }
}
