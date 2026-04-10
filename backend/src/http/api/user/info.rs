use axum::{Extension, debug_handler, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use http::header;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    entity::{
        asset::{Asset, AssetHandle, AssetStatus},
        user::{user_info::UserInfo, user_profiles::UserProfile},
    },
    error::{
        CustomError,
        api::{ApiError, ApiErrorKind},
        db::DbError,
    },
    http::{
        api::{ApiResult, user::UserIdent},
        extractor::{Json, Path},
    },
    server::ServerState,
};

#[debug_handler]
#[tracing::instrument(name = "[user/info]", skip(state))]
pub(super) async fn get(State(state): State<ServerState>, Path(id): Path<Uuid>) -> ApiResult {
    let res = { UserProfile::fetch_all_fields_by_id(&state.database, id).await? };

    Ok((StatusCode::OK, axum::Json(res)).into_response())
}

#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "kebab-case")]
pub(super) enum PathParam {
    Avatar,
    Banner,
    Biography,
    Other,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct JsonBody {
    name: Option<String>,
    contact_me: Option<serde_json::Value>,
}

#[debug_handler]
#[tracing::instrument(name = "[user/info]", skip(state, info))]
pub(super) async fn put(
    State(state): State<ServerState>,
    Path(part): Path<PathParam>,
    Extension(ident): Extension<UserIdent>,
    info: Option<Json<JsonBody>>,
) -> ApiResult {
    if let PathParam::Other = part {
        return update_name_or_contact_method(state, ident.id, info).await;
    }

    let handle = AssetHandle::generate();

    let new_asset = Asset {
        id: handle.id,
        status: AssetStatus::Init,
        owner: ident.id,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };

    {
        let mut transacton = state.database.begin().await.map_err(DbError::from)?;

        let mut user_profile = UserProfile::fetch_all_fields_by_id(transacton.as_mut(), ident.id).await?;

        match part {
            PathParam::Avatar => user_profile.avatar = Some(handle),
            PathParam::Banner => user_profile.banner = Some(handle),
            PathParam::Biography => user_profile.biography = Some(handle),
            PathParam::Other => unreachable!(),
        }
        new_asset.insert(transacton.as_mut()).await?;

        if user_profile.write_back(transacton.as_mut()).await?.is_some() {
            let url = format!("/asset/{}", handle.id);
            transacton.commit().await.map_err(DbError::from)?;
            tracing::info!("insertion suceeded");
            Ok((
                StatusCode::CREATED,
                [(header::LOCATION, &url)],
                axum::Json(json!({
                    "id": new_asset.id,
                    "url": &url
                })),
            )
                .into_response())
        } else {
            tracing::warn!("this guy doesn't even exsists");
            Err(ApiError::new(ApiErrorKind::Unauthorized).into_response())
        }
    }
}

async fn update_name_or_contact_method(state: ServerState, id: Uuid, info: Option<Json<JsonBody>>) -> ApiResult {
    if let Some(Json(info)) = info {
        if let Some(name) = info.name {
            UserInfo::update_name(&state.database, id, &name).await?;
        }
        if let Some(contact_me) = info.contact_me {
            let mut user_profile = UserProfile::fetch_all_fields_by_id(&state.database, id).await?;
            user_profile.contact_me = contact_me;
            user_profile.write_back(&state.database).await?;
        }

        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Ok((StatusCode::NO_CONTENT).into_response())
    }
}
