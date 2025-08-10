use crate::entity::usr::usr_info::Model;
use crate::entity::usr::{prelude::UsrInfo, usr_info};
use crate::http::api::ApiResult;
use crate::http::api::usr::{Param, UsrIdent};
use crate::http::jwt::Jwt;
use crate::server::ServerState;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, debug_handler};
use axum_valid::Valid;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter, QueryTrait};

type LoginParam = Param;

#[debug_handler]
#[tracing::instrument(name = "[usr/login] by id", skip_all, fields(usr_id = %id))]
pub(crate) async fn login_by_id(
    state: State<ServerState>,
    Path(id): Path<i64>,
    passwd: String,
) -> ApiResult {
    let res = UsrInfo::find_by_id(id).one(state.db());

    check_passwd_and_respond(res.await, &passwd)
}

#[debug_handler]
#[tracing::instrument(name = "[usr/login] by phone/email", skip_all, fields(method = %param.method.get_anyway()))]
pub(super) async fn login_by_email_or_phone(
    state: State<ServerState>,
    param: Valid<Json<LoginParam>>,
) -> ApiResult {
    let method = &param.method;
    let res = UsrInfo::find()
        .apply_if(method.get_email(), |rows, email| {
            rows.filter(usr_info::Column::Email.eq(email))
        })
        .apply_if(method.get_phone(), |rows, phone| {
            rows.filter(usr_info::Column::Phone.eq(phone))
        })
        .one(state.db());

    Ok(check_passwd_and_respond(res.await, &param.passwd)?)
}

fn check_passwd_and_respond(query_res: Result<Option<Model>, DbErr>, passwd: &str) -> ApiResult {
    match query_res {
        Ok(Some(val)) => check_passwd(val, passwd),
        Ok(None) => {
            tracing::info!("No account for this user");
            Err(StatusCode::UNAUTHORIZED.into_response())
        }
        Err(e) => {
            tracing::error!("Database error! details: {e}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "You can't login now, because of database error, which is my fault.",
            )
                .into_response())
        }
    }
}

fn check_passwd(val: Model, passwd: &str) -> ApiResult {
    match argon2::verify_encoded(&val.passwd_hash, passwd.as_bytes()) {
        Ok(true) => {
            tracing::info!("User {} login successfully", val.id);
            Ok((
                StatusCode::OK,
                Jwt::generate(UsrIdent {
                    id: val.id,
                    name: val.name,
                    email: val.email,
                    phone: val.phone,
                }),
            )
                .into_response())
        }
        Ok(false) => {
            tracing::info!("User {} login with incrrect pasword", val.id);
            Err(StatusCode::UNAUTHORIZED.into_response())
        }
        Err(e) => {
            tracing::error!("Error checking password! {e}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error occurs while checking your password, which is my fault!",
            )
                .into_response())
        }
    }
}
