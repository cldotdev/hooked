mod config;
mod crypto;
mod logger;
mod trigger;
mod web;
use crate::config::{Command, Config};
use crate::logger::setup_logger;
use crate::web::app::start_server;

#[tokio::main]
async fn main() {
    let config = Config::load();

    if let Some(Command::Trigger { commit }) = &config.command {
        match trigger::run(&config, commit).await {
            Ok(message) => println!("{message}"),
            Err(e) => {
                eprintln!("error: {e:#}");
                std::process::exit(1);
            }
        }
        return;
    }

    let _guard = setup_logger(&config.log_file_path).expect("failed to set up logger");

    start_server(config).await.expect("failed to start server");
}
