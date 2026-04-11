use axum::{
    debug_handler,
    extract::{Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use lettre::message::{Mailbox, header::ContentType};
use redis::AsyncCommands;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;
use validator::{Validate, ValidationError, ValidationErrors};

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
        extractor::ValidJson,
        utils::{validate_email, validate_password_complexity, validate_password_length, validate_phone},
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignUpParam {
    name: String,
    #[serde(flatten)]
    method: SignUpMethod,
    password: String,
}

fn get_insert_param_key(id: Uuid) -> String {
    format!("brain-overflow:signup:{}", id)
}

fn get_confirm_code_key(id: Uuid) -> String {
    format!("brain-overflow:signup:{}:code", id)
}

#[debug_handler]
#[tracing::instrument(name = "[user/signup/confirm]", skip(state))]
pub(super) async fn confirm(State(state): State<ServerState>, Path((id, code)): Path<(Uuid, i32)>) -> ApiResult {
    // 获取验证码并比对
    let new_user = {
        let mut redis = state.redis.clone();

        // 有四种情况：获取验证码时出错了、没有这个验证码、验证码和提交的不匹配、验证码和提交的匹配
        match redis.get::<_, Option<i32>>(get_confirm_code_key(id)).await {
            Err(e) => {
                tracing::warn!("error while accessing redis: {e}");
                return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
            }
            Ok(None) => return Err(StatusCode::NOT_FOUND.into_response()),
            Ok(Some(v)) if v != code => return Err(StatusCode::UNAUTHORIZED.into_response()),
            _ => (),
        }

        match redis.get(get_insert_param_key(id)).await {
            Err(e) => {
                tracing::warn!("error while accessing redis: {e}");
                return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
            }
            Ok(None) => return Err(StatusCode::NOT_FOUND.into_response()),
            Ok(Some(v)) => v,
        }
    };

    {
        // 往数据库中添加 user_info 记录和 user_profile 记录
        let mut transacton = state.begin_transaction().await?;
        let id = UserInfo::insert_and_return_id(transacton.as_mut(), &new_user).await?;
        UserProfile::insert(id, transacton.as_mut()).await?;
        transacton.commit().await.map_err(DbError::from)?;
    }

    let user_ident = UserIdent {
        id,
        name: new_user.name,
        email: new_user.email,
        phone: new_user.phone,
    };

    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, state.prefix_uri(format!("/user/{}", id)))],
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

#[debug_handler]
#[tracing::instrument(name = "[user/signup/gen]", skip_all, fields(verify = %method.get_anyway()))]
pub(super) async fn signup(
    State(state): State<ServerState>,
    ValidJson(SignUpParam { name, method, password }): ValidJson<SignUpParam>,
) -> ApiResult {
    let (phone, email) = method.get_tup_phone_email();
    let password_hash = generate_password_hash(&password).await?;
    let new_user = InsertParam {
        id: Uuid::now_v7(),
        email,
        phone,
        name,
        password: password_hash,
    };

    // 生成六位验证码并将其写入 redis
    let confirm_code = format!("{:6}", rand::random_range(0..1_000_000));
    let (mut redis1, mut redis2) = (state.redis.clone(), state.redis.clone());

    match tokio::join!(
        redis1.set_ex::<_, _, ()>(get_insert_param_key(new_user.id), &new_user, 300), // 生成的新用户信息,
        redis2.set_ex::<_, _, ()>(get_confirm_code_key(new_user.id), &confirm_code, 300)  // 验证码
    ) {
        (Ok(_), Err(e)) | (Err(e), Ok(_)) => {
            tracing::warn!("error while accessing redis: {e}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        }
        (Err(e1), Err(e2)) => {
            tracing::warn!("error while accessing redis: {e1} {e2}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        }
        _ => {
            if let Some(email) = new_user.email {
                state.email_to(move |mail| {
                    mail.to(Mailbox::new(Some(new_user.name), email.parse().unwrap()))
                        .header(ContentType::TEXT_PLAIN)
                        .subject("Brain Overflow")
                        .body(format!("You are trying to sign up, your confirm code: {confirm_code}"))
                        .unwrap()
                });
            }
            Ok((StatusCode::OK, axum::Json(serde_json::json!({"id": new_user.id}))).into_response())
        }
    }
}

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

impl Validate for SignUpParam {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        use SignUpMethod::*;
        use std::borrow::Cow::*;
        let mut errors = ValidationErrors::new();

        if self.name.chars().count() > 32 {
            let mut e = ValidationError::new("length").with_message(Borrowed("your name is too long"));
            e.add_param(Borrowed("name"), &self.name);
            errors.add("name", e)
        }

        let _ = match &self.method {
            Email(email) => validate_email(email).map_err(|mut e| {
                e.add_param(Borrowed("email"), email);
                errors.add("email", e);
            }),
            Phone(phone) => validate_phone(phone).map_err(|mut e| {
                e.add_param(Borrowed("phone"), phone);
                errors.add("phone", e);
            }),
        };

        let _ = validate_password_length(&self.password).map_err(|mut e| {
            e.add_param(Borrowed("password"), &"it'a secret");
            errors.add("length", e);
        });

        let _ = validate_password_complexity(&self.password).map_err(|mut e| {
            e.add_param(Borrowed("password"), &"it'a secret");
            errors.add("complexity", e);
        });

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
