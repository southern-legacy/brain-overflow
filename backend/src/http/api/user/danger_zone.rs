use std::borrow::Cow;

use crate::{
    error::db::DbError,
    http::{
        api::{ApiResult, user::generate_password_hash},
        utils::{validate_email, validate_password, validate_phone},
    },
};
use axum::{Extension, debug_handler, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::{
    entity::user::user_info::UserInfo,
    http::{
        api::user::{UserIdent, check_password},
        extractor::ValidJson,
    },
    server::ServerState,
};

#[debug_handler]
#[tracing::instrument(name = "[user/delete_account]", skip_all, fields(user_id = %ident.id))]
pub(super) async fn delete_account(
    state: State<ServerState>,
    ident: Extension<UserIdent>,
    password: String,
) -> ApiResult {
    let user_info = ident.retrieve_self_from_db(state.database.as_ref()).await?;
    check_password(&user_info, &password).await?;
    try_delete_account(&state.database, ident.id).await
}

async fn try_delete_account(db: &PgPool, id: Uuid) -> ApiResult {
    let res = UserInfo::delete_by_id(db, id).await;
    match res {
        Ok(id) => {
            tracing::info!("User (id: {id}) deleted his/her account forever");
            Ok(StatusCode::NO_CONTENT.into_response())
        }
        Err(e) => {
            if e.is_not_found() {
                tracing::info!("Someone wants to delete user (id: {id}), which doesn't exists.");
                Err(StatusCode::UNAUTHORIZED.into_response())
            } else {
                tracing::error!(
                    "Error occurs while handling the unregister attempt of user (id: {id}), thus this should be a fatal error, details {e}."
                );
                Err(StatusCode::UNAUTHORIZED.into_response())
            }
        }
    }
}

#[derive(Deserialize, Validate)]
#[validate(schema(function = "Self::validate"))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ChangeAuthParam {
    #[validate(custom(function = "validate_email"))]
    new_email: Option<String>,
    #[validate(custom(function = "validate_phone"))]
    new_phone: Option<String>,
    #[validate(custom(function = "validate_password"))]
    new_password: Option<String>,

    password: String,
}

impl ChangeAuthParam {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut count = 0;
        if self.new_email.is_some() {
            count += 1;
        }
        if self.new_phone.is_some() {
            count += 1;
        }
        if self.new_password.is_some() {
            count += 1;
        }
        if count == 1 {
            Ok(())
        } else {
            Err(ValidationError::new("params")
                .with_message(Cow::Borrowed("excepted ONE field to change at ONE time")))
        }
    }
}

#[debug_handler]
#[tracing::instrument(name = "[user/change auth info]", skip_all, fields(user_id = %ident.id))]
pub(super) async fn change_auth_info(
    state: State<ServerState>,
    ident: Extension<UserIdent>,
    param: ValidJson<ChangeAuthParam>,
) -> ApiResult {
    let user_info = ident.retrieve_self_from_db(state.database.as_ref()).await?;

    let ChangeAuthParam {
        new_email,
        new_phone,
        new_password,
        password,
    } = param.unwrap();

    check_password(&user_info, &password).await?;

    let new_password_hash = match &new_password {
        Some(val) => Some(generate_password_hash(val).await?),
        None => None,
    };

    try_change_auth_info(
        &state,
        ident.id,
        new_email.as_ref(),
        new_phone.as_ref(),
        new_password_hash.as_ref(),
    )
    .await
}

async fn try_change_auth_info(
    state: &ServerState,
    id: Uuid,
    new_email: Option<&String>,
    new_phone: Option<&String>,
    new_password_hash: Option<&String>,
) -> ApiResult {
    let res = {
        let mut transacton = state.database.begin().await.map_err(DbError::from)?;

        let res = if let Some(new_email) = new_email {
            UserInfo::update_email(transacton.as_mut(), id, new_email).await
        } else if let Some(new_phone) = new_phone {
            UserInfo::update_phone(transacton.as_mut(), id, new_phone).await
        } else if let Some(new_password_hash) = new_password_hash {
            UserInfo::update_password_hash(transacton.as_mut(), id, new_password_hash).await
        } else {
            // 这里应该是 unreachable 的
            // return Err(StatusCode::UNPROCESSABLE_ENTITY.into_response())
            // 因为在上面的 ChangeAuthParam 的 Validate 实现中，保证了每次只能修改一条信息
            unreachable!()
        };

        transacton.commit().await.map_err(DbError::from)?;
        res
    };

    match res {
        Ok(res) => {
            let ident = UserIdent::from(res);
            Ok((
                StatusCode::OK,
                json!({
                    "id": ident.id,
                    "name": ident.name,
                    "phone": ident.phone,
                    "email": ident.email,
                    "token": ident.into_jwt(&state.config.auth.encoder_config)?
                })
                .to_string(),
            )
                .into_response())
        }
        Err(e) => {
            if e.is_not_found() {
                Err(StatusCode::UNAUTHORIZED.into_response())
            } else {
                Err(e.into_response())
            }
        }
    }
}
