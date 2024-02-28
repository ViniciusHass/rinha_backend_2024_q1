use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use time::OffsetDateTime;

#[derive(Clone, Serialize, Deserialize)]
pub struct Transaction {
    #[serde(rename = "valor")]
    pub value: i64,
    #[serde(rename = "tipo")]
    pub _type: String,
    #[serde(rename = "descricao")]
    pub description: String,
    #[serde(rename = "realizada_em")]
    pub timestamp: String,
}

impl Transaction {
    pub fn new(value: i64, _type: String, description: String, timestamp: String) -> Self {
        Transaction {
            value,
            _type,
            description,
            timestamp,
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct TransactionInput {
    #[serde(rename = "valor")]
    pub value: i64,
    #[serde(rename = "tipo")]
    pub _type: TransactionType,
    #[serde(rename = "descricao")]
    pub description: Decription,
}

impl TransactionInput {
    pub fn signed_value(&self) -> i64 {
        match self._type {
            TransactionType::Credit => self.value,
            TransactionType::Debit => -self.value,
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

impl From<Decription> for String {
    fn from(description: Decription) -> Self {
        description.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TransactionType {
    #[serde(rename = "c")]
    Credit,
    #[serde(rename = "d")]
    Debit,
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionType::Credit => write!(f, "c"),
            TransactionType::Debit => write!(f, "d"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TransactionAnswer {
    #[serde(rename = "limite")]
    pub limit: i64,
    #[serde(rename = "saldo")]
    pub current: i64,
}
