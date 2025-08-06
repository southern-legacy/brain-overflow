use std::net::Ipv4Addr;

use axum::handler::HandlerWithoutStateExt;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, 8080))
        .await
        .unwrap();

    axum::serve(listener, service.into_service()).await.unwrap()
}

async fn service() -> &'static str {
    "Hello world!"
}
