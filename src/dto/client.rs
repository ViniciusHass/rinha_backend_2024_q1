use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use super::transaction::{Transaction, TransactionAnswer, TransactionInput, TransactionType};

#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Client {
    #[serde(rename = "limite")]
    pub limit_value: i64,
    #[serde(rename = "total")]
    pub current: i64,
    #[serde(rename = "data_extrato", default = "Vec::new")]
    pub transactions_list: Vec<Transaction>,
}

impl Client {
    pub fn new(limit_value: i64, current: i64, transactions_list: Vec<Transaction>) -> Self {
        Client {
            limit_value,
            current,
            transactions_list,
        }
    }

    pub fn validate_transaction(
        &mut self,
        transaction: &TransactionInput,
    ) -> Result<TransactionAnswer, StatusCode> {
        match transaction._type {
            TransactionType::Credit => {
                self.current += transaction.value;
                Ok(TransactionAnswer {
                    limit: self.limit_value,
                    current: self.current,
                })
            }
            TransactionType::Debit => {
                if -self.limit_value <= self.current - transaction.value {
                    self.current -= transaction.value;
                    Ok(TransactionAnswer {
                        limit: self.limit_value,
                        current: self.current,
                    })
                } else {
                    Err(StatusCode::UNPROCESSABLE_ENTITY)
                }
            }
        }
    }

    pub fn list_information(&self) -> Result<Json<Value>, ()> {
        Ok(Json(json!({
            "saldo": {
                "total": self.current,
                "data_extrato": OffsetDateTime::now_utc().format(&Rfc3339).unwrap(),
                "limite": self.limit_value,
              },
              "ultimas_transacoes": self.transactions_list,

        })))
    }
}
