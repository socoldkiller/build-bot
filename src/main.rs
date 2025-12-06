mod config;
mod napbot;
mod tty;
mod version;
mod websocket;

use crate::config::{CliArgs, Config};
use crate::websocket::{HandleResult, WebSocketClient, WebSocketHandler};
use clap::Parser;

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

    if args.show_config {
        println!("Current configuration:");
        println!("WebSocket URL: {}", config.websocket_url);
        println!(
            "Access token: {}",
            config.access_token.as_deref().unwrap_or("None")
        );
        println!(
            "Log level: {}",
            config.log_level.as_deref().unwrap_or("info")
        );
        if let Some(reconnect) = &config.reconnect {
            println!("Reconnect settings:");
            println!("  Max attempts: {}", reconnect.max_attempts.unwrap_or(5));
            println!("  Delay seconds: {}", reconnect.delay_seconds.unwrap_or(5));
            println!(
                "  Backoff factor: {}",
                reconnect.backoff_factor.unwrap_or(1.5)
            );
        }
        return Ok(());
    }

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
    let mut client = WebSocketClient::connect(url).await?;

    let handler = WebSocketHandler::new(config.clone());
    println!("Connected! Waiting for NapCatResponse messages...");
    println!("Press Ctrl+C to exit.");

    let max_attempts = config
        .reconnect
        .as_ref()
        .and_then(|r| r.max_attempts)
        .unwrap_or(5);

    let mut attempt = 0;

    loop {
        match client.recv().await {
            Ok(message) => {
                match handler.handle_message(&message).await {
                    HandleResult::Error(_e) => {
                    }

                    HandleResult::Message(msg) => {
                        client.send(&msg).await?;
                    }
                    HandleResult::TtyFailed(msg) => {
                        client.send(&msg).await?;
                    }
                    HandleResult::NotForThisBot(_msg) =>{
                        // what can I say?
                    }
                    HandleResult::BrokenPipe(msg) => {
                        client.send(&msg).await?;
                    },
                    HandleResult::CreateTty(msg) => {
                        client.send(&msg).await?;
                    },
                }

                attempt = 0;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                attempt += 1;

                if attempt >= max_attempts {
                    eprintln!(
                        "Max reconnection attempts ({}) reached. Exiting.",
                        max_attempts
                    );
                    break;
                }

                let delay = calculate_reconnect_delay(attempt, config);
                eprintln!(
                    "Attempting to reconnect in {} seconds (attempt {}/{})...",
                    delay, attempt, max_attempts
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;

                match WebSocketClient::connect(url).await {
                    Ok(new_client) => {
                        client = new_client;
                        println!("Reconnected successfully!");
                    }
                    Err(e) => {
                        eprintln!("Reconnection failed: {}", e);
                    }
                }
            }
        }
    }

    client.close().await?;
    println!("Disconnected");
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

fn calculate_reconnect_delay(attempt: u32, config: &Config) -> u64 {
    let base_delay = config
        .reconnect
        .as_ref()
        .and_then(|r| r.delay_seconds)
        .unwrap_or(5);

    let backoff_factor = config
        .reconnect
        .as_ref()
        .and_then(|r| r.backoff_factor)
        .unwrap_or(1.5);

    (base_delay as f64 * backoff_factor.powi(attempt as i32 - 1)) as u64
}
