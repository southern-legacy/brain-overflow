use clap::Parser;
use tokio::{
    select,
    signal::{self, unix::SignalKind},
};

use crate::cli::Cli;

mod app_config;
mod cli;
mod database;
mod entity;
mod error;
mod http;
mod logger;
mod redis;
mod server;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let gracefully_shutdown = tokio::spawn(async {
        let ctrl_c = tokio::spawn(async {
            if signal::ctrl_c().await.is_err() {
                tracing::error!("Failed to install ctrl_c signal handle");
            } else {
                tracing::info!("Receiced ctrl_c, gracefully shutdown.");
            }
        });

        if !cfg!(windows) {
            let sigterm = match signal::unix::signal(SignalKind::terminate()) {
                Ok(mut sig) => tokio::spawn(async move {
                    sig.recv().await;
                    tracing::info!("Receiced SIGTERM, gracefully shutdown.");
                }),
                Err(e) => tokio::spawn(async move {
                    tracing::error!("Failed to install SIGTERM signal handle {}", e);
                }),
            };
            select! {
                _ = ctrl_c => {},
                _ = sigterm => {},
            };
        } else {
            ctrl_c.await.unwrap()
        }
    });

    select! {
        _ = gracefully_shutdown => {},
        _ = server::start(&cli) => {}
    };
}
