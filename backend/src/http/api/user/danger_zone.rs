use std::borrow::Cow;

use crate::{error::db::DbError, http::{
    api::{ApiResult, user::generate_passwd_hash},
    utils::{validate_email, validate_passwd, validate_phone},
}};
use axum::{Extension, debug_handler, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::{
    entity::user::user_info::UserInfo,
    http::{
        api::{
            user::{UserIdent, check_passwd},
        },
        extractor::ValidJson,
    },
    server::ServerState,
};

#[debug_handler]
#[tracing::instrument(name = "[user/delete_account]", skip_all, fields(user_id = %ident.id))]
pub(super) async fn delete_account(
    state: State<ServerState>,
    ident: Extension<UserIdent>,
    passwd: String,
) -> ApiResult {
    let user_info = ident.retrieve_self_from_db(&state.database).await?;
    check_passwd(&user_info, &passwd).await?;
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
    #[validate(custom(function = "validate_passwd"))]
    new_passwd: Option<String>,

    passwd: String,
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
        if self.new_passwd.is_some() {
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
#[tracing::instrument(name = "[user/delete_account]", skip_all, fields(user_id = %ident.id))]
pub(super) async fn change_auth_info(
    state: State<ServerState>,
    ident: Extension<UserIdent>,
    param: ValidJson<ChangeAuthParam>,
) -> ApiResult {
    let user_info = ident.retrieve_self_from_db(&state.database).await?;

    let ChangeAuthParam {
        new_email,
        new_phone,
        new_passwd,
        passwd,
    } = param.unwrap();

    check_passwd(&user_info, &passwd).await?;

    let new_passwd_hash = match &new_passwd {
        Some(val) => Some(generate_passwd_hash(val).await?),
        None => None,
    };

    try_change_auth_info(
        &state,
        ident.id,
        new_email.as_ref(),
        new_phone.as_ref(),
        new_passwd_hash.as_ref(),
    )
    .await
}

async fn try_change_auth_info(
    state: &ServerState,
    id: Uuid,
    new_email: Option<&String>,
    new_phone: Option<&String>,
    new_passwd_hash: Option<&String>,
) -> ApiResult {
    let mut tx = state.database.begin().await.map_err(DbError::from)?;

    let res;
    if let Some(new_email) = new_email {
        res = UserInfo::update_email(tx.as_mut(), id, new_email).await;
    } else if let Some(new_phone) = new_phone {
        res = UserInfo::update_phone(tx.as_mut(), id, new_phone).await;
    } else if let Some(new_passwd_hash) = new_passwd_hash {
        res = UserInfo::update_passwd_hash(tx.as_mut(), id, new_passwd_hash).await;
    } else {
        // 这里应该是 unreachable 的
        // return Err(StatusCode::UNPROCESSABLE_ENTITY.into_response())
        // 因为在上面的 ChangeAuthParam 的 Validate 实现中，保证了每次只能修改一条信息
        unreachable!()
    }

    tx.commit().await.map_err(DbError::from)?;

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
                }).to_string(),
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
