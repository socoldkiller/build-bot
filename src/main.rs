mod napbot;
mod tty;
mod websocket;
use crate::websocket::{WebSocketClient, WebSocketHandler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_napbot_websocket_client("xxx").await
}

async fn run_napbot_websocket_client(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = WebSocketClient::connect(url).await?;

    let handler = WebSocketHandler::new();
    println!("Connected! Waiting for NapCatResponse messages...");
    println!("Press Ctrl+C to exit.");
    loop {
        match client.recv().await {
            Ok(message) => {
                match handler.handle_message(&message).await {
                    Ok(output) => {
                        client.send(output.as_str()).await?;
                    }

                    Err(_) => {
                    }
                }
            }

            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }

    client.close().await?;
    println!("Disconnected");
    Ok(())
}
