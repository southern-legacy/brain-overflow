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
    let mut tx = state.database.begin().await.map_err(DbError::from)?;

    let res = UserProfile::fetch_all_fields_by_id(tx.as_mut(), id).await?;
    tx.commit().await.map_err(DbError::from)?;

    Ok((StatusCode::OK, axum::Json(res)).into_response())
}

#[derive(Deserialize, Clone, Copy)]
pub(super) enum PathParam {
    Avatar,
    Banner,
    Biography,
}

#[debug_handler]
pub(super) async fn put(
    State(state): State<ServerState>,
    Path(part): Path<PathParam>,
    Extension(ident): Extension<UserIdent>,
) -> ApiResult {
    let handle = AssetHandle::generate(OwnerType::User);
    let url = format!("/assets/{}", Uuid::new_v4());
    let new_asset = Asset {
        id: handle.id,
        newest_key: url.clone(),
        owner: ident.id,
        status: AssetStatus::Uploading,
        owner_type: OwnerType::User,
        history: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };

    {
        let mut tx = state.database.begin().await.map_err(DbError::from)?;

        let mut user_profile = UserProfile::fetch_all_fields_by_id(tx.as_mut(), ident.id).await?;
        match part {
            PathParam::Avatar => user_profile.avatar = Some(handle),
            PathParam::Banner => user_profile.banner = Some(handle),
            PathParam::Biography => user_profile.biography = Some(handle),
        }

        new_asset.write_back(&state.database).await?;

        if user_profile.write_back(tx.as_mut()).await?.is_some() {
            Ok((StatusCode::CREATED, [(header::LOCATION, url)]).into_response())
        } else {
            Err(ApiError::new(ApiErrorKind::Unauthorized).into_response())
        }
    }
}
