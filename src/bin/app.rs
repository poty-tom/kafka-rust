use axum::{
    http::StatusCode, response::IntoResponse, routing::{get, post}, Json, Router
};
use axum::extract::State;
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;
use anyhow::{Context, Result};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::config::ClientConfig;
use rdkafka::util::Timeout;
use std::sync::Arc;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String
}

#[derive(Clone)]
pub struct AppRegistry {
    pub procedure: Arc<FutureProducer>
}

impl AppRegistry {
    pub fn new() -> Self {
        let procedure: Arc<FutureProducer> = Arc::new(ClientConfig::new()
        .set("bootstrap.servers", "localhost:9094")
        .set("messge.timeout.ms", "1000")
        .create()
        .expect("Unexpected error when creating Kafka Producer."));

        Self {
            procedure
        }
    }

    pub fn kafka_procedure(&self) -> Arc<FutureProducer> {
        self.procedure.clone()
    }
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
    let app_registry = AppRegistry::new();
    let app = Router::new()
        .route("/", get(health_check))
        .route("/login", post(login_with_kafka))
        .with_state(app_registry);
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

// ヘルスチェック
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

// Kafka簡易Producer
pub async fn login_with_kafka(
    State(registry): State<AppRegistry>,
    Json(req): Json<LoginRequest>
) -> impl IntoResponse {
    // ログイン
    // unimplemented!()
    // ログインリクエスト(req.email)をKafkaにプロデュース
    registry.kafka_procedure().send(FutureRecord::<(), _>::to("my-topic")
        .payload(&req.email), Timeout::Never)
        .await
        .expect("failed to produce");
    StatusCode::OK
}