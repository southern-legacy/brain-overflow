use std::borrow::Cow;

use crate::http::{
    api::usr::generate_passwd_hash,
    utils::{validate_email, validate_passwd, validate_phone},
};
use axum::{Extension, debug_handler, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use sqlx::PgPool;
use validator::{Validate, ValidationError};

use crate::{
    entity::usr::usr_info::UsrInfo,
    http::{
        api::{
            ApiResult,
            usr::{UsrIdent, check_passwd},
        },
        extractor::ValidJson,
    },
    server::ServerState,
};

#[debug_handler]
#[tracing::instrument(name = "[usr/delete_account]", skip_all, fields(usr_id = %ident.id))]
pub(super) async fn delete_account(
    state: State<ServerState>,
    ident: Extension<UsrIdent>,
    passwd: String,
) -> ApiResult {
    let usr_info = ident.retrieve_self_from_db(state.db()).await?;
    check_passwd(&usr_info, &passwd).await?;
    try_delete_account(state.db(), ident.id).await
}

async fn try_delete_account(db: &PgPool, id: i64) -> ApiResult {
    let res = UsrInfo::delete_by_id(db, id).await;
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
#[tracing::instrument(name = "[usr/delete_account]", skip_all, fields(usr_id = %ident.id))]
pub(super) async fn change_auth_info(
    state: State<ServerState>,
    ident: Extension<UsrIdent>,
    param: ValidJson<ChangeAuthParam>,
) -> ApiResult {
    let usr_info = ident.retrieve_self_from_db(state.db()).await?;

    let ChangeAuthParam {
        new_email,
        new_phone,
        new_passwd,
        passwd,
    } = param.unwrap();

    check_passwd(&usr_info, &passwd).await?;

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
    id: i64,
    new_email: Option<&String>,
    new_phone: Option<&String>,
    new_passwd_hash: Option<&String>,
) -> ApiResult {
    let res;
    if let Some(new_email) = new_email {
        res = UsrInfo::update_email(state.db(), id, new_email).await;
    } else if let Some(new_phone) = new_phone {
        res = UsrInfo::update_phone(state.db(), id, new_phone).await;
    } else if let Some(new_passwd_hash) = new_passwd_hash {
        res = UsrInfo::update_passwd_hash(state.db(), id, new_passwd_hash).await;
    } else {
        // 这里应该是 unreachable 的
        // return Err(StatusCode::UNPROCESSABLE_ENTITY.into_response())
        unreachable!()
    }

    match res {
        Ok(res) => {
            if res.id == id {
                Ok(UsrIdent::from(res).issue_as_jwt(state.jwt_config()))
            } else {
                unreachable!()
            }
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
