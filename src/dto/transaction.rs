use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use std::convert::TryFrom;

#[derive(Clone, Serialize, Deserialize)]
pub struct Transaction {
    #[serde(rename = "valor")]
    pub value: i32,
    #[serde(rename = "tipo")]
    pub _type: String,
    #[serde(rename = "descricao")]
    pub description: String,
    #[serde(rename = "realizada_em")]
    pub timestamp: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TransactionInput {
    #[serde(rename = "valor")]
    pub value: i32,
    #[serde(rename = "tipo")]
    pub _type: TransactionType,
    #[serde(rename = "descricao")]
    pub description: Decription,
    #[serde(
        rename = "realizada_em",
        with = "time::serde::rfc3339",
        default = "OffsetDateTime::now_utc"
    )]
    pub timestamp: OffsetDateTime,
}

impl Transaction {
    pub fn new(
        value: i32,
        _type: String,
        description: String,
        timestamp: String,
    ) -> Self {
        Transaction {
            value,
            _type,
            description,
            timestamp,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct Decription(String);

impl TryFrom<String> for Decription {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() || value.len() > 10 {
            Err("String invalida")
        } else {
            Ok(Self(value))
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TransactionType {
    #[serde(rename = "c")]
    Credit,
    #[serde(rename = "d")]
    Debit,
}

#[derive(Clone, Serialize)]
pub struct TransactionAnswer {
    #[serde(rename = "limite")]
    pub limit: i32,
    #[serde(rename = "saldo")]
    pub current: i32,
}
