mod app_config;
mod cli;
mod db;
mod entity;
mod error;
mod http;
mod logger;
mod server;

#[tokio::main]
async fn main() {
    server::start().await;
}
