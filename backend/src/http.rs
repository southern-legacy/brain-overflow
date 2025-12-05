mod api;
pub mod auth;
mod extractor;
mod middleware;
mod services;
mod utils;

use std::sync::LazyLock;

pub use api::build_router;
use crab_vault::auth::JwtEncoder;

use crate::{app_config, error::cli::MultiCliError};

const ENCODER_TO_CRAB_VAULT: LazyLock<JwtEncoder> = LazyLock::new(|| {
    app_config::auth()
        .encoder_config_to_crab_vault()
        .clone()
        .try_into()
        .map_err(MultiCliError::exit_now)
        .unwrap()
});

const ENCODER_TO_SELF: LazyLock<JwtEncoder> = LazyLock::new(|| {
    app_config::auth()
        .encoder_to_self()
        .clone()
        .try_into()
        .map_err(MultiCliError::exit_now)
        .unwrap()
});
