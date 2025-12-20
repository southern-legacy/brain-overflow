mod app_config;
mod cli;
mod database;
mod entity;
mod error;
mod http;
mod logger;
mod server;

#[tokio::main]
async fn main() {
    server::start().await;
}

// struct AsyncRunner<F> {
//     handler: F,
// }

// impl<F> AsyncRunner<F>
// where
//     F: AsyncFn(&str) -> usize,
// {
//     async fn run(&self, data: &str) {
//         let len = (self.handler)(data).await;
//         println!("处理后的长度: {}", len);
//     }
// }

// #[tokio::main]
// async fn main() {
//     let runner = AsyncRunner {
//         handler: async |s: &str| s.len(),
//     };
    
//     runner.run("Hello Rust 1.85").await;
// }