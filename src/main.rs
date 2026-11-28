use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::{EnvFilter, fmt};

mod config;
mod discovery;
mod error;
mod server;
mod ssh_config;
mod state;
mod tools;
mod transport;

use server::SshConnectServer;

#[tokio::main]
async fn main() -> Result<()> {
    // MCP uses stdout for JSON-RPC — all logging must go to stderr
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting SSH-Connect MCP Server");

    let server = SshConnectServer::new();

    let service = server
        .serve(stdio())
        .await
        .inspect_err(|e| tracing::error!("Server error: {:?}", e))?;

    service.waiting().await?;
    Ok(())
}
