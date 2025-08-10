use axum::{Extension, debug_handler, extract::State, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;

use crate::{
    entity::usr::usr_info::UsrInfo, http::api::{
        usr::{danger_zone_auth, UsrIdent}, ApiResult
    }, server::ServerState
};

#[debug_handler]
#[tracing::instrument(name = "[usr/signup]", skip_all, fields(usr_id = %ident.id))]
pub(super) async fn delete_account(
    state: State<ServerState>,
    ident: Extension<UsrIdent>,
    passwd: String,
) -> ApiResult {
    let _usr_info = danger_zone_auth(state.db(), &ident, passwd).await?;
    Ok(try_delete_account(state.db(), ident.id).await?)
}

async fn try_delete_account(db: &PgPool, id: i64) -> ApiResult {
    match UsrInfo::delete_by_id(db, id).await {
        Ok(res) => {
            if res > 1 {
                unreachable!()
            } else if res == 1 {
                tracing::info!("User deleted his/her account forever");
                Ok((StatusCode::OK, "Your account has been deleted forever!").into_response())
            } else {
                tracing::info!("Seems like the deletion is not successful.");
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Your account can't be removed by now.",
                )
                    .into_response())
            }
        }
        Err(e) => {
            tracing::error!(
                "Fail to delete the user because of database error, details: {}",
                e
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Your account can't be removed by now.",
            )
                .into_response())
        }
    }
}
