use axum::{debug_handler, extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use crate::{entity::usr::prelude::UsrInfo, http::middelware::auth::UsrIdent, server::ServerState};

#[debug_handler]
pub(super) async fn delete_account(
    state: State<ServerState>,
    ident: Extension<UsrIdent>,
    passwd: Json<String>,
) -> impl IntoResponse {
    let res = UsrInfo::find_by_id(ident.id).one(state.db()).await;
    match res {
        Ok(Some(val)) => {
            tracing::info!("Found the specified account!");
            match argon2::verify_encoded(&val.passwd_hash, passwd.as_bytes()) {
                Ok(true) => {
                    match val.into_active_model().delete(state.db()).await {
                        Ok(res) => {
                            if res.rows_affected > 1 {
                                unreachable!()
                            } else if res.rows_affected == 1 {
                                tracing::info!("User deleted his/her account forever");
                                return (StatusCode::OK, "Your account has been deleted forever!")
                            } else {
                                tracing::info!("Seems like the deletion is not successful.");
                                return (StatusCode::INTERNAL_SERVER_ERROR, "Your account can't be removed by now.");
                            }
                        },
                        Err(e) => {
                            tracing::error!("Fail to delete the user because of database error, details: {}", e);
                            return (StatusCode::INTERNAL_SERVER_ERROR, "Your account can't be removed by now.")
                        }
                    }
                },
                Ok(false) => {
                    tracing::info!("User intended to delete the account with incrrect pasword");
                    return (StatusCode::UNAUTHORIZED, "Your password and account don't match")
                }
                Err(e) => {
                    tracing::error!("Error checking password! {e}");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Error occurs while checking your account info! Please change your pasword!"
                    )
                }
            }
        },
        Ok(None) => {
            tracing::info!("Seems like there's no user in database.");
            return (StatusCode::NOT_FOUND, "No account specified found in database");
        },
        Err(err) => {
            tracing::error!("Failed to find the user in the database! {}", err);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Can't locate your account!");
        }
    }
}