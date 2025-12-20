use std::borrow::Cow;

use crate::{
    entity::user::user_info::UserInfo,
    http::{
        api::{
            ApiResult,
            user::{UserIdent, check_passwd},
        },
        extractor::ValidJson,
        utils::{validate_email, validate_passwd, validate_phone},
    },
    server::ServerState,
};

use axum::{debug_handler, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum LoginMethod {
    Id(Uuid),
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
#[tracing::instrument(name = "[user/login]", skip_all, fields(login_method = %param.method.get_anyway()))]
pub(super) async fn login(
    State(state): State<ServerState>,
    ValidJson(param): ValidJson<LoginParam>,
) -> ApiResult {
    let method = &param.method;
    let res = match method {
        LoginMethod::Phone(phone) => UserInfo::fetch_all_fields_by_phone(&state.database, phone).await,
        LoginMethod::Email(email) => UserInfo::fetch_all_fields_by_email(&state.database, email).await,
        LoginMethod::Id(id) => UserInfo::fetch_all_fields_by_id(&state.database, *id).await,
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

    check_passwd(&res, &param.passwd).await?;

    let user = UserIdent::from(res);

    Ok((
        StatusCode::OK,
        json!({
            "id": user.id,
            "name": user.name,
            "email": user.email,
            "phone": user.phone,
            "token": user.into_jwt(&state.config.auth.encoder_config)?
        }).to_string(),
    )
        .into_response())
}


impl LoginMethod {
    fn get_anyway(&self) -> Cow<'_, str> {
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
