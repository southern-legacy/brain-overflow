use axum::{
    Json, debug_handler,
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use axum_valid::Valid;
use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use sea_orm::{
    ActiveValue::{NotSet, Set},
    EntityTrait, IntoActiveModel,
};
use serde::Deserialize;
use validator::Validate;

use crate::http::{
    api::usr::{ARGON2_CONFIG, UsrIdent},
    jwt::Jwt,
};
use crate::server::ServerState;
use crate::{
    entity::usr::{
        prelude::UsrInfo,
        usr_info
    },
    http::api::usr::{LoginMethod, Param},
};

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
) -> Response {
    let salt = BASE64_STANDARD_NO_PAD.encode(uuid::Uuid::new_v4().into_bytes());
    let SignupParam { name, usr_param } = param;
    let Param { method, passwd } = usr_param;

    let (email, phone) = match method.clone() {
        LoginMethod::Email(email) => (Set(Some(email)), NotSet),
        LoginMethod::Phone(phone) => (NotSet, Set(Some(phone))),
    };

    let new_usr = usr_info::ActiveModel {
        id: NotSet,
        email: email,
        phone: phone,
        name: Set(name.clone()),
        salt: Set(salt.clone()),
        passwd_hash: match argon2::hash_encoded(passwd.as_bytes(), salt.as_bytes(), &ARGON2_CONFIG)
        {
            Ok(val) => Set(val),
            Err(e) => {
                tracing::error!("Error occured while hashing the password! {e}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        },
    };

    match UsrInfo::insert(new_usr.into_active_model())
        .exec(state.db())
        .await
    {
        Ok(val) => {
            tracing::info!("Successfully insert a user into database.");
            (
                StatusCode::CREATED,
                [(header::LOCATION, format!("/usr/{}", val.last_insert_id))],
                Jwt::generate(UsrIdent {
                    id: val.last_insert_id,
                    email: method.get_email().map(|v| String::from(v)),
                    phone: method.get_phone().map(|v| String::from(v)),
                    name,
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to sign up a user! details: {:#?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
