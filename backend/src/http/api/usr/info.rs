use axum::{extract::State, response::{IntoResponse, Response}, Extension};

use crate::{http::api::usr::UsrIdent, server::ServerState};

pub(super) async fn info(
    state: State<ServerState>,
    Extension(ident): Extension<UsrIdent>,
) -> Result<Response, Response> {
    Ok(axum::Json(ident.retreive_self_from_db(state.db()).await?).into_response())
}