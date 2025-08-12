pub mod usr;

use axum::{
    Router,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    error::{DbError, ViolationKind}, server::ServerState
};

type ApiResult = Result<Response, Response>;

impl From<DbError> for Response {
    fn from(value: DbError) -> Self {
        use DbError::*;
        match value {
            Unprocessable(e) => {
                tracing::error!("Error occurs while manipulating database! Details: {e}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            Violation(e) => {
                use ViolationKind::*;
                use tracing::warn;
                match &e {
                    Unique(error) => warn!("Unique key violation! Details: {}", error.message()),
                    Foreign(error) => warn!("Foreign key violation! Details: {}", error.message()),
                    Check(error) => warn!("Check key violation! Details: {}", error.message()),
                    NotNull(error) => warn!("Not null key violation! Details: {}", error.message()),
                    Other(error) =>  warn!("Other violation! Details: {}", error.message()),
                };
                (StatusCode::UNPROCESSABLE_ENTITY, axum::Json(e)).into_response()
            }
            NotFound => StatusCode::NOT_FOUND.into_response()
        }
    }
}

pub fn build_router() -> Router<ServerState> {
    Router::new()
        .nest("/usr", usr::build_router())
        .fallback(|| async { StatusCode::NOT_FOUND })
        .method_not_allowed_fallback(|| async { StatusCode::METHOD_NOT_ALLOWED })
}
