use axum::{Router, routing::get, http::StatusCode};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;
use anyhow::{Context, Result};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub async fn handler() -> StatusCode {
    StatusCode::OK
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger()?;
    startup().await
}

fn init_logger() -> Result<()> {
    let subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_target(false);

    tracing_subscriber::registry()
        .with(subscriber)
        .try_init()?;

    Ok(())
}

async fn startup() -> Result<()> {
    let app = Router::new()
        .route("/", get(handler));

    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080);
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Listening on {}", addr);
    axum::serve(listener, app)
        .await
        .context("Unexpected error happend in server")
        .inspect_err(|e| {
            tracing::error!(
                error.cause_chain = ?e,error.message = %e, "Unexpected error"
            )
        }) 
}