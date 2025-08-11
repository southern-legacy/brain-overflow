use axum::{
    Json, debug_handler,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
};
use axum_valid::Valid;
use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use serde::Deserialize;
use validator::Validate;

use crate::{
    entity::usr::usr_info::{InsertParam, UsrInfo},
    http::{
        api::{usr::{Param, UsrIdent, ARGON2_CONFIG}, ApiResult},
        jwt::Jwt,
    }
};
use crate::server::ServerState;

#[derive(Deserialize, Validate)]
pub(super) struct SignupParam {
    #[validate(length(max = 32))]
    name: String,

    #[validate(nested)]
    #[serde(flatten)]
    usr_param: Param,
}

#[debug_handler]
#[tracing::instrument(name = "[usr/signup]", skip_all)]
pub(super) async fn signup(
    state: State<ServerState>,
    Valid(Json(param)): Valid<Json<SignupParam>>,
) -> ApiResult {
    let salt = BASE64_STANDARD_NO_PAD.encode(uuid::Uuid::new_v4().into_bytes());
    let SignupParam { name, usr_param } = param;
    let Param { method, passwd } = usr_param;

    let (phone, email) = method.get_tup_phone_email();

    let new_usr = InsertParam {
        email: email,
        phone: phone,
        name: name,
        passwd: match argon2::hash_encoded(passwd.as_bytes(), salt.as_bytes(), &ARGON2_CONFIG)
        {
            Ok(val) => val,
            Err(e) => {
                tracing::error!("Error occured while hashing the password! {e}");
                return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
            }
        }
    };

    let val =  UsrInfo::insert_and_return_all(state.db(), new_usr).await?;
    tracing::info!("Successfully insert a user into database.");
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, format!("/usr/{}", val.id))],
        Jwt::generate(UsrIdent {
            id: val.id,
            name: val.name,
            email: val.email,
            phone: val.phone,
        }),
    )
        .into_response())
}
