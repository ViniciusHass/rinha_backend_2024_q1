use crate::dto::TransactionInput;

use super::super::dto::{Client, Transaction, TransactionAnswer};
use sqlx::postgres::PgQueryResult;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
pub struct PostgresRepository {
    pool: PgPool,
}
use axum::http::StatusCode;
type ResultRepository<T> = Result<T, StatusCode>;

impl PostgresRepository {
    pub async fn connect(url: &str, pool_size: u32) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new().max_connections(2).connect(url).await?;

        Ok(PostgresRepository { pool })
    }

    pub async fn get_transactions(&self, id: i32) -> ResultRepository<Client> {
        let query = format!(
            r#"
            SELECT
                limit_value,
                current,
                COALESCE(
                    (
                        SELECT
                            jsonb_agg(
                                jsonb_strip_nulls(
                                    jsonb_build_object(
                                        'valor', t.value,
                                        'tipo', t.type,
                                        'descricao', t.description,
                                        'realizada_em', t.timestamp
                                    )
                                )
                            )
                        FROM
                            (
                                SELECT
                                    t.value,
                                    t.type,
                                    t.description,
                                    t.timestamp
                                FROM
                                    transactions t
                                WHERE
                                    t.client_id = clientes.id
                                ORDER BY
                                    t.client_id,
                                    t.timestamp DESC
                                LIMIT 10
                            ) t
                    ), '[]'::jsonb
                ) as transactions_list
            FROM
                clientes
            WHERE
                clientes.id = $1;
            "#
        );

        let result = sqlx::query(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await;

        let client = match result {
            Ok(Some(row)) => {
                let limit_value: i64 = row.try_get("limit_value").map_err(|e| {
                    println!("Error decoding 'limit_value': {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
                let current: i64 = row.try_get("current").map_err(|e| {
                    println!("Error decoding 'current': {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

    
                let transactions_list_result: Result<Option<Vec<Transaction>>, _> = {
                    let transactions_list_json: serde_json::Value =
                        row.try_get("transactions_list").unwrap_or_default();
                    serde_json::from_value(transactions_list_json)
                };
    
                let transactions_list: Vec<Transaction> = transactions_list_result.and_then(|list| {
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
                }).map_err(|e| {
                    println!("Error decoding 'transactions_list': {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
    
                Ok(Client::new(limit_value, current, transactions_list))
            }
            Ok(None) => {
                println!("Client not found for id {}", id);
                Err(StatusCode::NOT_FOUND)
            }
            Err(e) => {
                println!("Error fetching transactions for client {}: {:?}", id, e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        };

        return client;
    }

    pub async fn get_client_info(&self, id: i32) -> ResultRepository<Client> {
        let result = sqlx::query(
            "
            SELECT
                limit_value,
                current
            FROM
                clientes
            WHERE
                clientes.id = $1
            ",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);

        let client = match result {
            Ok(Some(row)) => {
                let limit_value: i64 = row
                    .try_get("limit_value")
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                let current: i64 = row
                    .try_get("current")
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                Ok(Client::new(limit_value, current, Vec::new()))
            }
            Ok(None) => Err(StatusCode::NOT_FOUND),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        };
        return client;
    }

    pub async fn push_transaction(
        &self,
        id: i32,
        transaction: TransactionInput,
        answer: TransactionAnswer,
    ) -> Result<TransactionAnswer, StatusCode> {
        let mut transaction_db = self
            .pool
            .begin()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let description_str: String = transaction.clone().description.try_into().unwrap();

        let insert_statement_transactions = "
        INSERT INTO transactions (client_id, value, type, description, timestamp)
        SELECT $1, $2, $3, $4, CURRENT_TIMESTAMP;
        ";

        sqlx::query(insert_statement_transactions)
            .bind(id.clone())
            .bind(transaction.value)
            .bind(transaction._type.to_string())
            .bind(description_str)
            .execute(&mut *transaction_db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let insert_statement_client = "
            UPDATE clientes SET current = $1 WHERE id = $2
            ";

        sqlx::query(insert_statement_client)
            .bind(&answer.current)
            .bind(id.clone())
            .execute(&mut *transaction_db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        transaction_db
            .commit()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        return Ok(answer);
    }
}
