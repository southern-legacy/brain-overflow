use axum::{
    debug_handler,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;
use validator::{Validate, ValidationErrors};

use crate::{
    entity::user::{
        user_info::{InsertParam, UserInfo},
        user_profiles::UserProfile,
    },
    error::db::DbError,
    http::{
        api::{
            ApiResult,
            user::{UserIdent, generate_password_hash},
        },
        extractor::prelude::ValidJson,
        utils::{validate_email, validate_password, validate_phone},
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
pub struct SignUpParam {
    #[validate(length(max = 32))]
    name: String,

    #[validate(nested)]
    #[serde(flatten)]
    method: SignUpMethod,

    #[validate(custom(function = "validate_password"))]
    password: String,
}

#[debug_handler]
#[tracing::instrument(name = "[user/signup]", skip_all, fields(verify = %param.method.get_anyway()))]
pub(super) async fn signup(
    State(state): State<ServerState>,
    ValidJson(param): ValidJson<SignUpParam>,
) -> ApiResult {
    let SignUpParam {
        name,
        method,
        password,
    } = param;

    let (phone, email) = method.get_tup_phone_email();

    let password_hash = generate_password_hash(&password).await?;

    let new_user = InsertParam {
        email: email.as_ref(),
        phone: phone.as_ref(),
        name: &name,
        password: &password_hash,
    };

    let id = {
        // 往数据库中添加 user_info 记录和 user_profile 记录
        let mut transacton = state.database.begin().await.map_err(DbError::from)?;
        let id = UserInfo::insert_and_return_id(transacton.as_mut(), new_user).await?;
        UserProfile::insert(id, transacton.as_mut()).await?;
        transacton.commit().await.map_err(DbError::from)?;
        id
    };

    tracing::info!("Successfully inserted a user into database.");

    let user_ident = UserIdent {
        id,
        name,
        email,
        phone,
    };

    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, format!("/user/{}", id))],
        json!({
            "id": user_ident.id,
            "name": user_ident.name,
            "email": user_ident.email,
            "phone": user_ident.phone,
            "token": user_ident.into_jwt(&state.config.auth.encoder_config)?
        })
        .to_string(),
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
