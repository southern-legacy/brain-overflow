use crate::entity::usr::usr_info::UsrInfo;
use crate::http::api::ApiResult;
use crate::http::api::usr::{check_passwd, LoginMethod, Param, UsrIdent};
use crate::http::jwt::Jwt;
use crate::server::ServerState;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, debug_handler};
use axum_valid::Valid;

type LoginParam = Param;

#[debug_handler]
#[tracing::instrument(name = "[usr/login] by id", skip_all, fields(usr_id = %id))]
pub(crate) async fn login_by_id(
    state: State<ServerState>,
    Path(id): Path<i64>,
    passwd: String,
) -> ApiResult {
    let res = UsrInfo::find_by_id(state.db(), id).await?;
    Ok(check_passwd_and_respond(res, &passwd).await?)
}

#[debug_handler]
#[tracing::instrument(name = "[usr/login] by phone/email", skip_all, fields(method = %param.method.get_anyway()))]
pub(super) async fn login_by_email_or_phone(
    state: State<ServerState>,
    Valid(Json(param)): Valid<Json<LoginParam>>,
) -> ApiResult {
    let method = &param.method;
    let res = match method {
        LoginMethod::Phone(phone) => UsrInfo::find_by_phone(state.db(), phone).await?,
        LoginMethod::Email(email) => UsrInfo::find_by_email(state.db(), email).await?,
    };
    
    Ok(check_passwd_and_respond(res, &param.passwd).await?)
}

async fn check_passwd_and_respond(res: Option<UsrInfo>, passwd: &str) -> ApiResult {
    let usr = match res {
        Some(usr) => {
            check_passwd(&usr, passwd).await?;
            usr
        },
        None => return Err(StatusCode::UNAUTHORIZED.into_response())
    };
    
    Ok((StatusCode::OK, Jwt::generate(UsrIdent {
        email: usr.email,
        phone: usr.phone,
        id: usr.id,
        name: usr.name
    })).into_response())
}