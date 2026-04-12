use axum::{Extension, debug_handler, extract::State, response::IntoResponse};
use http::StatusCode;

use crate::{
    http::api::{
        ApiResult,
        user::{AccessToken, RefreshToken},
    },
    server::ServerState,
};

#[debug_handler]
pub(super) async fn refresh(State(state): State<ServerState>, Extension(ident): Extension<RefreshToken>) -> ApiResult {
    let config = &state.config().auth.refresh;
    Ok((
        StatusCode::OK,
        axum::Json(serde_json::json!({"token": AccessToken::from(ident.id).into_jwt(config)?})),
    )
        .into_response())
}
