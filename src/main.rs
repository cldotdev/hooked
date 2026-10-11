mod config;
mod crypto;
mod logger;
mod web;
use crate::config::Config;
use crate::logger::setup_logger;
use crate::web::app::start_server;

#[tokio::main]
async fn main() {
    let config = Config::load();
    let _guard = setup_logger(&config.log_file_path).expect("failed to set up logger");

    start_server(config).await.expect("failed to start server");
}
