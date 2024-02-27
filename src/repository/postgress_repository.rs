use super::super::dto::{Client, Transaction};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
pub struct PostgresRepository {
    pool: PgPool,
}
use axum::http::StatusCode;
type ResultRepository<T> = Result<T, StatusCode>;

use serde_json::Error as SerdeError;

#[derive(Debug)]
pub enum CustomError {
    Serde(SerdeError),
    StatusCode(StatusCode),
}

impl From<SerdeError> for CustomError {
    fn from(error: SerdeError) -> Self {
        CustomError::Serde(error)
    }
}

impl From<StatusCode> for CustomError {
    fn from(status_code: StatusCode) -> Self {
        CustomError::StatusCode(status_code)
    }
}

impl PostgresRepository {
    pub async fn connect(url: &str, pool_size: u32) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(pool_size)
            .connect(url)
            .await?;

        Ok(PostgresRepository { pool })
    }

    pub async fn get_client(&self, id: i32) -> ResultRepository<Client> {
        let query = format!(
            r#"
            SELECT
                clientes.id as id,
                limit_value as limit_value,
                current,
                COALESCE(
                    (
                        SELECT jsonb_agg(
                            jsonb_strip_nulls(
                                jsonb_build_object(
                                    'valor', t.value,
                                    'tipo', t.type,
                                    'descricao', t.description,
                                    'realizada_em', t.timestamp
                                )
                            )
                        ) 
                        FROM (
                            SELECT
                                t.value,
                                t.type,
                                t.description,
                                t.timestamp
                            FROM transactions t
                            WHERE t.client_id = clientes.id
                            ORDER BY t.client_id, t.timestamp DESC
                            LIMIT 10
                        ) t
                    ),
                    '[]'::jsonb
                ) as transactions_list
            FROM
                clientes
            WHERE
                clientes.id = $1;
            FOR NO KEY UPDATE
            "#
        );

        let result = sqlx::query(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await;

        let client = match result {
            Ok(Some(row)) => {
                let limit_value: i32 = row
                    .try_get("limit_value")
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                let current: i32 = row
                    .try_get("current")
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                let transactions_list_result: Result<Option<Vec<Transaction>>, _> = {
                    let transactions_list_json: serde_json::Value =
                        row.try_get("transactions_list").unwrap_or_default();
                    serde_json::from_value(transactions_list_json)
                };

                let transactions_list: Vec<Transaction> = transactions_list_result
                    .and_then(|list| {
                        list.unwrap_or_default()
                            .into_iter()
                            .map(|trans| {
                                Ok(Transaction {
                                    value: trans.value,
                                    _type: trans._type,
                                    description: trans.description,
                                    timestamp: trans.timestamp,
                                })
                            })
                            .collect::<Result<Vec<_>, _>>()
                    })
                    .unwrap_or_default();

                Ok(Client::new(limit_value, current, transactions_list))
            }
            Ok(None) => Err(StatusCode::NOT_FOUND),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        };

        return client;
    }
}
