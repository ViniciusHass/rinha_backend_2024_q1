mod dto;
mod repository;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use env_logger::Env;
use log::{error, info};
use std::{env, sync::Arc};

use self::dto::TransactionInput;
use self::repository::PostgresRepository;

type AppState = Arc<PostgresRepository>;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .filter_level(match env::var("LOG_LEVEL") {
            Ok(log) => match log.as_str() {
                "trace" => log::LevelFilter::Trace,
                "debug" => log::LevelFilter::Debug,
                "info" => log::LevelFilter::Info,
                "warn" => log::LevelFilter::Warn,
                "error" => log::LevelFilter::Error,
                _ => log::LevelFilter::Info,
            },
            Err(_) => log::LevelFilter::Info,
        })
        .init();

    info!("Starting server...");

    let database_url = env::var("DATABASE_URL")
        .unwrap_or(String::from("postgres://admin:123@localhost:5432/rinha"));

    let repo = PostgresRepository::connect(&database_url, 2).await.unwrap();
    info!("Connected to the database.");

    let app_state = Arc::new(repo);

    let app = Router::new()
        .route("/health", get(health))
        .route("/clientes/:id/extrato", get(list_transactions))
        .route("/clientes/:id/transacoes", post(create_transaction))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app)
        .await
        .map_err(|e| error!("Server error: {:?}", e))
        .unwrap();
    info!("Server stopped.");
}

async fn health() -> impl IntoResponse {
    info!("Health check performed.");
    Json(&"OK")
}

async fn list_transactions(
    Path(user_id): Path<u8>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.get_transactions(user_id.into()).await {
        Ok(client) => {
            info!("Successfully fetched transactions for client {}", user_id);
            Ok(client.list_information())
        }
        Err(e) => {
            error!(
                "Error fetching transactions for client {}: {:?}",
                user_id, e
            );
            Err(e)
        }
    }
}

async fn create_transaction(
    Path(user_id): Path<u8>,
    State(state): State<AppState>,
    Json(transaction): Json<TransactionInput>,
) -> impl IntoResponse {
    let mut client = state.get_client_info(user_id.clone().into()).await?;
    let answer = client.validate_transaction(&transaction)?;
    match state
        .push_transaction(user_id.into(), transaction, answer)
        .await
    {
        Ok(answer) => {
            info!("Transaction created for client {}: {:?}", user_id, answer);
            Ok(Json(answer))
        }
        Err(e) => {
            error!("Error creating transaction for client {}: {:?}", user_id, e);
            Err(e)
        }
    }
}
