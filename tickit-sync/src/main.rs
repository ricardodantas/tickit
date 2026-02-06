//! tickit-sync - Self-hosted sync server for Tickit
//!
//! A minimal sync server that stores tasks, lists, and tags,
//! enabling sync across multiple Tickit clients.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod api;
mod config;
mod db;
mod models;

use config::Config;

#[derive(Parser)]
#[command(name = "tickit-sync")]
#[command(about = "Self-hosted sync server for Tickit task manager")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the sync server
    Serve {
        /// Config file path
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Port to listen on (overrides config)
        #[arg(short, long)]
        port: Option<u16>,

        /// Bind address (overrides config)
        #[arg(short, long)]
        bind: Option<String>,
    },

    /// Generate a new API token
    Token {
        /// Optional name/label for the token
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Initialize a new config file
    Init {
        /// Output path for config file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("tickit_sync=info".parse().unwrap()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { config, port, bind } => {
            let mut cfg = if let Some(path) = config {
                Config::load_from(&path)?
            } else {
                Config::load()?
            };

            // Override with CLI args
            if let Some(p) = port {
                cfg.server.port = p;
            }
            if let Some(b) = bind {
                cfg.server.bind = b;
            }

            run_server(cfg).await
        }

        Commands::Token { name } => {
            let token = generate_token();
            let label = name.unwrap_or_else(|| "default".to_string());

            println!("Generated API token for '{}':", label);
            println!();
            println!("  {}", token);
            println!();
            println!("Add this to your config.toml:");
            println!();
            println!("  [[tokens]]");
            println!("  name = \"{}\"", label);
            println!("  token = \"{}\"", token);
            println!();
            println!("Then configure Tickit client with:");
            println!();
            println!("  [sync]");
            println!("  enabled = true");
            println!("  server = \"http://your-server:3030\"");
            println!("  token = \"{}\"", token);

            Ok(())
        }

        Commands::Init { output } => {
            let path = output.unwrap_or_else(|| PathBuf::from("config.toml"));
            let cfg = Config::default();
            cfg.save_to(&path)?;

            println!("Created config file: {}", path.display());
            println!();
            println!("Next steps:");
            println!("  1. Generate a token: tickit-sync token --name my-device");
            println!("  2. Add the token to config.toml");
            println!(
                "  3. Start the server: tickit-sync serve --config {}",
                path.display()
            );

            Ok(())
        }
    }
}

async fn run_server(config: Config) -> Result<()> {
    let db = db::Database::open(&config.database.path).context("Failed to open database")?;

    let state = api::AppState::new(db, config.clone());
    let app = api::create_router(state);

    let addr = format!("{}:{}", config.server.bind, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("🚀 tickit-sync server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

fn generate_token() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    let bytes: [u8; 32] = rng.random();

    // Base64-like encoding but URL-safe
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    bytes
        .iter()
        .map(|b| ALPHABET[(*b as usize) % ALPHABET.len()] as char)
        .collect()
}
