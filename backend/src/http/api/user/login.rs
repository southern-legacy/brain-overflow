use std::borrow::Cow;

use crate::{
    entity::user::user_info::UserInfo,
    error::db::DbError,
    http::{
        api::{
            ApiResult,
            user::{UserIdent, check_password},
        },
        extractor::Json,
    },
    server::ServerState,
};

use axum::{debug_handler, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum LoginMethod {
    Id(Uuid),
    Email(String),
    Phone(String),
}

#[derive(Deserialize)]
pub(super) struct LoginParam {
    #[serde(flatten)]
    method: LoginMethod,

    password: String,
}

#[debug_handler]
#[tracing::instrument(name = "[user/login]", skip_all, fields(login_method = %param.method.get_anyway()))]
pub(super) async fn login(
    State(state): State<ServerState>,
    Json(param): Json<LoginParam>,
) -> ApiResult {
    let method = &param.method;

    let res = {
        // 我们先查找数据库中的记录
        let mut transacton = state.database.begin().await.map_err(DbError::from)?;
        let res = match method {
            LoginMethod::Phone(num) => {
                UserInfo::fetch_all_fields_by_phone(transacton.as_mut(), num).await
            }
            LoginMethod::Email(add) => {
                UserInfo::fetch_all_fields_by_email(transacton.as_mut(), add).await
            }
            LoginMethod::Id(id) => UserInfo::fetch_all_fields_by_id(transacton.as_mut(), *id).await,
        };
        transacton.commit().await.map_err(DbError::from)?;
        res
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

    check_password(&res, &param.password).await?;

    let user = UserIdent::from(res);

    Ok((
        StatusCode::OK,
        json!({
            "id": user.id,
            "name": user.name,
            "email": user.email,
            "phone": user.phone,
            "token": user.into_jwt(&state.config.auth.encoder_config)?
        })
        .to_string(),
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
