mod dto;
mod utils;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use self::dto::{Client, Transaction};

type AccountsState = Arc<HashMap<u8, RwLock<Client>>>;

#[tokio::main]
async fn main() {
    // TODO: Connect to the bank
    let accounts = HashMap::<u8, RwLock<Client>>::from_iter([
        (1, RwLock::new(Client::create_with_value(100_000))),
        (2, RwLock::new(Client::create_with_value(80_000))),
        (3, RwLock::new(Client::create_with_value(1_000_000))),
        (4, RwLock::new(Client::create_with_value(10_000_000))),
        (5, RwLock::new(Client::create_with_value(500_000))),
    ]);

    // build our application with a route
    let app = Router::new()
        .route("/clientes/:id/extrato", get(list_transactions))
        .route("/clientes/:id/transacoes", post(create_transaction))
        .with_state(Arc::new(accounts));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn list_transactions(
    Path(user_id): Path<u8>,
    State(accounts_states): State<AccountsState>,
) -> impl IntoResponse {
    match accounts_states.get(&user_id) {
        Some(client) => Ok(client.read().unwrap().list_infomation()),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_transaction(
    Path(user_id): Path<u8>,
    State(accounts_states): State<AccountsState>,
    Json(transaction): Json<Transaction>,
) -> impl IntoResponse {
    match accounts_states.get(&user_id) {
        Some(client) => {
            let mut _client = client.write().unwrap();
            match _client.push_transaction(transaction) {
                Ok(value) => Ok(Json(value)),
                Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
            }
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}
