mod app_config;
mod db;
mod entity;
mod logger;
mod server;
mod http;

#[tokio::main]
async fn main() {
    server::start().await;
}
