use axum::{Extension, debug_handler, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use http::header;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    entity::{
        asset::{Asset, AssetHandle, AssetStatus, OwnerType},
        user::user_profiles::UserProfile,
    },
    error::{
        CustomError,
        api::{ApiError, ApiErrorKind},
        db::DbError,
    },
    http::{
        api::{ApiResult, user::UserIdent},
        extractor::Path,
    },
    server::ServerState,
};

#[debug_handler]
#[tracing::instrument(name = "[user/info]", skip(state))]
pub(super) async fn get(State(state): State<ServerState>, Path(id): Path<Uuid>) -> ApiResult {
    let res = {
        let res = UserProfile::fetch_all_fields_by_id(state.database.as_ref(), id).await?;
        res
    };

    Ok((StatusCode::OK, axum::Json(res)).into_response())
}

#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "kebab-case")]
pub(super) enum PathParam {
    Avatar,
    Banner,
    Biography,
}

#[debug_handler]
#[tracing::instrument(name = "[user/info]", skip(state))]
pub(super) async fn put(
    State(state): State<ServerState>,
    Path(part): Path<PathParam>,
    Extension(ident): Extension<UserIdent>,
) -> ApiResult {
    let handle = AssetHandle::generate(OwnerType::User);
    let url = format!("/assets/{}", handle.id);
    let new_asset = Asset {
        id: handle.id,
        newest_key: url.clone(),
        owner: ident.id,
        status: AssetStatus::Init,
        owner_type: OwnerType::User,
        history: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };
    tracing::info!("genetated an asset handle, preparing for inserting");

    {
        let mut transacton = state.database.begin().await.map_err(DbError::from)?;

        let mut user_profile = UserProfile::fetch_all_fields_by_id(transacton.as_mut(), ident.id).await?;
        match part {
            PathParam::Avatar => user_profile.avatar = Some(handle),
            PathParam::Banner => user_profile.banner = Some(handle),
            PathParam::Biography => user_profile.biography = Some(handle),
        }
        new_asset.insert(transacton.as_mut()).await?;
        if user_profile.write_back(transacton.as_mut()).await?.is_some() {
            transacton.commit().await.map_err(DbError::from)?;
            tracing::info!("insertion suceeded");
            Ok((StatusCode::CREATED, [(header::LOCATION, url)]).into_response())
        } else {
            tracing::warn!("this guy doesn't even exsists");
            Err(ApiError::new(ApiErrorKind::Unauthorized).into_response())
        }
    }
}
