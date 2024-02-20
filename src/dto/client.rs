use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use super::transaction::{Transaction, TransactionAnswer, TransactionType};
use super::super::utils::RingBuffer;

#[derive(Clone, Deserialize, Serialize)]
pub struct Client {
    #[serde(rename = "limite")]
    pub limit: i32,
    #[serde(rename = "total")]
    pub current: i32,
    #[serde(rename = "data_extrato")]
    pub transactions_list: RingBuffer<Transaction>,
}

impl Client {
    pub fn create_with_value(limit: i32) -> Self {
        Client {
            limit,
            current: 0,
            transactions_list: RingBuffer::new(),
        }
    }

    pub fn push_transaction(&mut self, transaction: Transaction) -> Result<TransactionAnswer, ()> {
        match transaction._type {
            TransactionType::Credit => {
                self.current += transaction.value;
                self.transactions_list.push(transaction);
                Ok(TransactionAnswer {
                    limit: self.limit,
                    current: self.current,
                })
            }
            TransactionType::Debit => {
                if -self.limit <= self.current - transaction.value {
                    self.current -= transaction.value;
                    self.transactions_list.push(transaction);
                    Ok(TransactionAnswer {
                        limit: self.limit,
                        current: self.current,
                    })
                } else {
                    Err(())
                }
            }
        }
    }

    pub fn list_infomation(&self) -> Result<Json<Value>, ()> {
        Ok(Json(json!({
            "saldo": {
                "total": self.current,
                "data_extrato": OffsetDateTime::now_utc().format(&Rfc3339).unwrap(),
                "limite": self.limit,
              },
              "ultimas_transacoes": self.transactions_list,

        })))
    }
}
