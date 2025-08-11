pub mod usr;

use axum::{http::StatusCode, response::{IntoResponse, Response}, routing, Router};

use crate::{db::{SqlxError, Violation}, server::ServerState};

type ApiResult = Result<Response, Response>;

impl From<SqlxError> for Response {
    fn from(value: SqlxError) -> Self {
        use SqlxError::*;
        use Violation::*;
        match value {
            Unprocessible(e) => {
                tracing::error!("Error occurs while manipulate database! Details: {e}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            },
            Processible(e) => {
                let e = match e {
                    Unique(error) => {
                        tracing::warn!("Unique key violation! Details: {}", error.message());
                        Unique(error)
                    },
                    Foreign(error) => {
                        tracing::warn!("Foreign key violation! Details: {}", error.message());
                        Foreign(error)
                    },
                    Check(error) => {
                        tracing::warn!("Check key violation! Details: {}", error.message());
                        Check(error)
                    },
                    NotNull(error) => {
                        tracing::warn!("Not null key violation! Details: {}", error.message());
                        NotNull(error)
                    },
                    Other(error) => {
                        tracing::warn!("Other violation! Details: {}", error.message());
                        Other(error)
                    },
                };
                (StatusCode::UNPROCESSABLE_ENTITY, axum::Json(e)).into_response()
            },
        }
    }
}

pub fn build_router() -> Router<ServerState> {
    Router::new()
        .nest("/usr", usr::build_router())
        .route("/test", routing::get(|| async { "Hello world!" }))
}
