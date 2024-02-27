use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use super::transaction::{Transaction, TransactionAnswer, TransactionType};

#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Client {
    #[serde(rename = "limite")]
    pub limit_value: i32,
    #[serde(rename = "total")]
    pub current: i32,
    #[serde(rename = "data_extrato")]
    pub transactions_list: Vec<Transaction>,
}

impl Client {
    pub fn new(limit_value: i32, current: i32, transactions_list: Vec<Transaction>) -> Self {
        Client {
            limit_value,
            current,
            transactions_list,
        }
    }

    pub fn push_transaction(&mut self, transaction: Transaction) -> Result<TransactionAnswer, ()> {
        match transaction._type {
            TransactionType::Credit => {
                self.current += transaction.value;
                self.transactions_list.push(transaction);
                Ok(TransactionAnswer {
                    limit: self.limit_value,
                    current: self.current,
                })
            }
            TransactionType::Debit => {
                if -self.limit_value <= self.current - transaction.value {
                    self.current -= transaction.value;
                    self.transactions_list.push(transaction);
                    Ok(TransactionAnswer {
                        limit: self.limit_value,
                        current: self.current,
                    })
                } else {
                    Err(())
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
