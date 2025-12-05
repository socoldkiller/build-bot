use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use toml;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// WebSocket server URL
    pub websocket_url: String,
    
    /// Optional authentication token
    pub access_token: Option<String>,
    
    /// Log level (debug, info, warn, error)
    pub log_level: Option<String>,
    
    /// Bot nickname for command matching (e.g., "bff")
    #[serde(default = "default_bot_nickname")]
    pub bot_nickname: String,
    
    /// Reconnection settings
    pub reconnect: Option<ReconnectConfig>,
}

fn default_bot_nickname() -> String {
    "bff".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectConfig {
    /// Maximum number of reconnection attempts
    pub max_attempts: Option<u32>,
    
    /// Delay between reconnection attempts in seconds
    pub delay_seconds: Option<u64>,
    
    /// Exponential backoff factor
    pub backoff_factor: Option<f64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            websocket_url: String::from("ws://localhost:3000"),
            access_token: None,
            log_level: Some(String::from("info")),
            bot_nickname: default_bot_nickname(),
            reconnect: Some(ReconnectConfig::default()),
        }
    }
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            max_attempts: Some(5),
            delay_seconds: Some(5),
            backoff_factor: Some(1.5),
        }
    }
}

impl Config {
    /// Load configuration from a file
    pub fn from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Save configuration to a file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Create configuration from CLI arguments
    pub fn from_cli_args(args: &CliArgs) -> Self {
        let mut config = if let Some(config_path) = &args.config_file {
            match Self::from_file(config_path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Warning: Failed to load config from {}: {}", config_path.display(), e);
                    Config::default()
                }
            }
        } else {
            Config::default()
        };
        
        // Override with CLI arguments if provided
        if let Some(url) = &args.url {
            config.websocket_url = url.clone();
        }
        
        if let Some(token) = &args.token {
            config.access_token = Some(token.clone());
        }
        
        if let Some(log_level) = &args.log_level {
            config.log_level = Some(log_level.clone());
        }
        
        config
    }
}

/// CLI arguments structure
#[derive(Debug, clap::Parser)]
#[command(name = "napbot")]
#[command(about = "NapBot WebSocket client", long_about = None)]
pub struct CliArgs {
    /// Path to configuration file (TOML format)
    #[arg(short = 'f', long = "config-file", value_name = "FILE", alias = "config")]
    pub config_file: Option<PathBuf>,
    
    /// WebSocket server URL
    #[arg(short, long, value_name = "URL")]
    pub url: Option<String>,
    
    /// Access token for authentication
    #[arg(short, long, value_name = "TOKEN")]
    pub token: Option<String>,
    
    /// Log level (debug, info, warn, error)
    #[arg(long, value_name = "LEVEL")]
    pub log_level: Option<String>,
    
    /// Generate a sample configuration file
    #[arg(long, value_name = "FILE")]
    pub generate_config: Option<PathBuf>,
    
    /// Show current configuration
    #[arg(long)]
    pub show_config: bool,
}
