mod dto;
mod repository;

use self::repository::PostgresRepository;
use std::{env, sync::Arc};

use self::dto::Transaction;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

type AppState = Arc<PostgresRepository>;

#[tokio::main]
async fn main() {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or(String::from("postgres://admin:123@localhost:5432/rinha"));

    let repo = PostgresRepository::connect(&database_url, 30)
        .await
        .unwrap();

    let app_state = Arc::new(repo);

    let app = Router::new()
        .route("/clientes/:id/extrato", get(list_transactions))
        // .route("/clientes/:id/transacoes", post(create_transaction))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn list_transactions(
    Path(user_id): Path<u8>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    
    match state.get_client(user_id.into()).await {
        Ok(client) => Ok(client.list_information()),
        Err(e) => Err(e),
    }
}

// async fn create_transaction(
//     Path(user_id): Path<u8>,
//     State(accounts_states): State<AccountsState>,
//     Json(transaction): Json<Transaction>,
// ) -> impl IntoResponse {
//     match accounts_states.get(&user_id) {
//         Some(client) => {
//             let mut _client = client.write().unwrap();
//             match _client.push_transaction(transaction) {
//                 Ok(value) => Ok(Json(value)),
//                 Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
//             }
//         }
//         None => Err(StatusCode::NOT_FOUND),
//     }
// }
