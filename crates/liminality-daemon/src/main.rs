//! liminality-daemon
//!
//! This crate is the runtime host for liminality.
//! For now, it provides a minimal CLI/server shell.
//! It depends on liminality-model, liminality-engine, and liminality-protocol.

use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use clap::{Parser, Subcommand};
use liminality_engine::simulate_furnace_line;
use liminality_model::WorldModel;
use liminality_protocol::{Query as ProtocolQuery, Response as ProtocolResponse, WorldSnapshot};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Demo {
        /// The name of the demo to run (e.g., "furnace-line")
        demo_name: String,
    },
}

struct AppState {
    model: Mutex<WorldModel>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let cli = Cli::parse();

    let model = match &cli.command {
        Commands::Demo { demo_name } => {
            if demo_name == "furnace-line" {
                info!("Initializing furnace-line demo...");
                WorldModel::furnace_line_demo()
            } else {
                anyhow::bail!("Unknown demo name: {}", demo_name);
            }
        }
    };

    let shared_state = Arc::new(AppState {
        model: Mutex::new(model),
    });

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/snapshot", get(snapshot_handler))
        .route("/query", post(query_handler))
        .route("/demo/furnace-line/state", get(demo_state_handler))
        .with_state(shared_state);

    let addr = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("Listening on http://{}", addr);
    info!("Auth is disabled only for local dev.");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

async fn snapshot_handler(State(state): State<Arc<AppState>>) -> Json<WorldSnapshot> {
    let model = {
        let lock = state.model.lock().unwrap();
        lock.clone()
    };
    Json(WorldSnapshot { state: model })
}

async fn query_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ProtocolQuery>,
) -> Json<ProtocolResponse> {
    let model = {
        let lock = state.model.lock().unwrap();
        lock.clone()
    };

    let result_state = simulate_furnace_line(&model, payload.seconds);

    Json(ProtocolResponse {
        state: WorldSnapshot {
            state: result_state,
        },
    })
}

#[derive(Deserialize)]
struct DemoStateQuery {
    seconds: u64,
}

async fn demo_state_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DemoStateQuery>,
) -> Json<WorldSnapshot> {
    let model = {
        let lock = state.model.lock().unwrap();
        lock.clone()
    };

    let result_state = simulate_furnace_line(&model, query.seconds);

    Json(WorldSnapshot {
        state: result_state,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app_for_test() -> Router {
        let model = WorldModel::furnace_line_demo();
        let shared_state = Arc::new(AppState {
            model: Mutex::new(model),
        });

        Router::new()
            .route("/health", get(health_handler))
            .route("/snapshot", get(snapshot_handler))
            .route("/query", post(query_handler))
            .route("/demo/furnace-line/state", get(demo_state_handler))
            .with_state(shared_state)
    }

    #[tokio::test]
    async fn test_health() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app_for_test()).await.unwrap();
        });

        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://{}/health", addr))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_snapshot() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app_for_test()).await.unwrap();
        });

        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://{}/snapshot", addr))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let snapshot: WorldSnapshot = response.json().await.unwrap();
        assert_eq!(snapshot.state.coal_storage, 32);
        assert_eq!(snapshot.state.ore_storage, 128);
        assert_eq!(snapshot.state.output_storage, 0);
    }

    #[tokio::test]
    async fn test_query_600s() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app_for_test()).await.unwrap();
        });

        let query = ProtocolQuery { seconds: 600 };
        let client = reqwest::Client::new();
        let response = client
            .post(format!("http://{}/query", addr))
            .json(&query)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let resp: ProtocolResponse = response.json().await.unwrap();
        assert_eq!(resp.state.state.output_storage, 60);
        assert_eq!(resp.state.state.ore_storage, 68);
        assert_eq!(resp.state.state.coal_storage, 24);
    }

    #[tokio::test]
    async fn test_demo_state_640s() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app_for_test()).await.unwrap();
        });

        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "http://{}/demo/furnace-line/state?seconds=640",
                addr
            ))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let snapshot: WorldSnapshot = response.json().await.unwrap();
        assert_eq!(snapshot.state.output_storage, 64);
        assert_eq!(snapshot.state.ore_storage, 64);
        assert_eq!(snapshot.state.coal_storage, 24);
    }
}
