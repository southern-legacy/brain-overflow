use axum::{Extension, debug_handler, extract::State, response::IntoResponse};

use crate::{
    http::api::{ApiResult, usr::UsrIdent},
    server::ServerState,
};

#[debug_handler]
#[tracing::instrument(name = "[usr/bio]", skip(state))]
pub(super) async fn bio_get(
    state: State<ServerState>,
    Extension(ident): Extension<UsrIdent>,
) -> ApiResult {
    Ok(axum::Json(ident.retreive_self_from_db(state.db()).await?).into_response())
}
