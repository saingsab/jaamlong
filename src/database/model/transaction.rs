use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct Transaction {
    pub id: Uuid,
    pub sender_address: String,
    pub receiver_address: String,
    pub from_token_address: String,
    pub to_token_address: String,
    pub origin_network: Option<Uuid>,
    pub destin_network: Option<Uuid>,
    pub asset_type: Option<Uuid>,
    pub transfer_amount: i64,
    pub bridge_fee: i64,
    pub tx_status: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


