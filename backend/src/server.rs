use std::net::{Ipv4Addr, Ipv6Addr};
use axum::debug_handler;
use tokio::net::TcpListener;
use tracing::info;
use crate::{app_config, http, logger};

pub async fn start() {
    logger::init();
    let _ = crate::db::init().await;

    let router = http::build_router();

    if app_config::get_server().ipv4_enabled() {
        let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, app_config::get_server().port()))
            .await
            .unwrap();
        info!("监听发往 {} 的请求", listener.local_addr().unwrap());
        axum::serve(listener, router).await.unwrap()
    } else if app_config::get_server().ipv6_enabled() {
        let listener = TcpListener::bind((Ipv6Addr::UNSPECIFIED, app_config::get_server().port()))
            .await
            .unwrap();
        info!("监听发往 {} 的请求", listener.local_addr().unwrap());
        axum::serve(listener, router).await.unwrap()
    }
}