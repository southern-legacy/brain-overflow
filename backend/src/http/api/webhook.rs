use std::str::FromStr;

use aws_lambda_events::s3::S3Event;
use axum::{Json, Router, debug_handler, extract::State, response::IntoResponse, routing};
use http::StatusCode;
use uuid::Uuid;

use crate::{
    entity::asset::{AssetHandle, AssetStatus},
    http::api::ApiResult,
    server::ServerState,
};

pub fn build_router() -> Router<ServerState> {
    Router::new().route("/s3/webhook", routing::post(s3_event_handler))
}

#[debug_handler]
async fn s3_event_handler(State(state): State<ServerState>, Json(event): Json<S3Event>) -> ApiResult {
    for record in event.records {
        if let Some(key) = record.s3.object.key {
            tracing::debug!(key, "handling webhook callback");
            if let Ok(id) = Uuid::from_str(&key)
                && let Err(e) = AssetHandle::new_with_id(id).set_status(AssetStatus::Available, &state.database).await
            {
                tracing::warn!("error while looking up the asset" = ?e);
            }
        }
    }

    Ok(StatusCode::OK.into_response())
}
