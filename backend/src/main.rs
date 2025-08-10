mod app_config;
mod db;
mod entity;
mod http;
mod logger;
mod server;

#[tokio::main]
async fn main() {
    server::start().await;
}
