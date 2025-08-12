use axum::{Extension, debug_handler, extract::State, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;

use crate::{
    entity::usr::usr_info::UsrInfo,
    http::api::{
        usr::{check_passwd, UsrIdent}, ApiResult
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
    let usr_info = ident.retreive_self_from_db(state.db()).await?;
    check_passwd(&usr_info, &passwd).await?;
    Ok(try_delete_account(state.db(), ident.id).await?)
}

async fn try_delete_account(db: &PgPool, id: i64) -> ApiResult {
    let res = UsrInfo::delete_by_id(db, id).await?;
    match res {
        Some(id) => {
            tracing::info!("User (id: {id}) deleted his/her account forever");
            Ok(StatusCode::NO_CONTENT.into_response())
        }
        None => {
            tracing::info!("Someone wants to delete user (id: {id}), which doesn't exists.");
            Err(StatusCode::UNAUTHORIZED.into_response())
        }
    }
}
