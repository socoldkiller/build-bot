mod config;
mod napbot;
mod tty;
mod version;
mod websocket;

use crate::config::{CliArgs, Config};
use crate::websocket::{HandleResult, WebSocketClient, WebSocketHandler};
use clap::Parser;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();

    if let Some(config_path) = &args.generate_config {
        let config = Config::default();
        config.save_to_file(config_path)?;
        println!("Sample configuration saved to: {}", config_path.display());
        return Ok(());
    }

    let config = Config::from_cli_args(&args);

    let url = build_websocket_url(&config);

    run_napbot_websocket_client(&url, &config).await
}

fn build_websocket_url(config: &Config) -> String {
    let mut url = config.websocket_url.clone();
    if !url.ends_with("/") {
        url.push_str("/");
    }

    if let Some(token) = &config.access_token {
        if url.contains('?') {
            url.push_str(&format!("&access_token={}", token));
        } else {
            url.push_str(&format!("?access_token={}", token));
        }
    }

    url
}

async fn run_napbot_websocket_client(
    url: &str,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    setup_logging(config);
    println!("Connecting to: {}", url);
    let c = Arc::new(Mutex::new(WebSocketClient::connect(url).await?));
    let cloned_c = Arc::clone(&c);
    let handler = WebSocketHandler::new(config.clone(), c);
    println!("Connected! Waiting for NapCatResponse messages...");
    println!("Press Ctrl+C to exit.");

    loop {
        let resp = handler.recv_message().await;

        if let HandleResult::NotForThisBot(e) = resp {
            continue;
        }

        if let HandleResult::Error(e) = resp {
            continue;
        }

        println!("Received response: {:?}", resp);
        let op = handler.send_message(resp).await;
        match op {
            HandleResult::SendError => break,
            _ => {}
        }
    }

    match cloned_c.lock().await.close().await {
        Ok(_) => println!("Disconnected"),
        Err(e) => println!("Error: {}", e),
    }

    Ok(())
}

fn setup_logging(config: &Config) {
    let log_level = config.log_level.as_deref().unwrap_or("info");

    match log_level.to_lowercase().as_str() {
        "debug" => println!("Log level set to: DEBUG"),
        "info" => println!("Log level set to: INFO"),
        "warn" => println!("Log level set to: WARN"),
        "error" => println!("Log level set to: ERROR"),
        _ => println!("Log level set to: INFO (default)"),
    }
}
