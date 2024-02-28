mod dto;
mod repository;

use self::repository::PostgresRepository;
use std::{env, sync::Arc};

use self::dto::{Transaction, TransactionInput};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use dto::{Client, TransactionAnswer};
use serde_json::json;

type AppState = Arc<PostgresRepository>;

#[tokio::main]
async fn main() {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or(String::from("postgres://admin:123@localhost:5432/rinha"));

    let repo = PostgresRepository::connect(&database_url, 2)
        .await
        .unwrap();

    let app_state = Arc::new(repo);

    let app = Router::new()
        .route("/health", get(health))
        .route("/clientes/:id/extrato", get(list_transactions))
        .route("/clientes/:id/transacoes", post(create_transaction))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> impl IntoResponse {
    Json(&"OK")
}

async fn list_transactions(
    Path(user_id): Path<u8>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.get_transactions(user_id.into()).await {
        Ok(client) => {
            println!("Successfully fetched transactions for client {}", user_id);
            Ok(client.list_information())
        }
        Err(e) => {
            println!(
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
        Ok(answer) => Ok(Json(answer)),
        Err(e) => Err(e),
    }
}
