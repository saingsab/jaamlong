use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgQueryResult, FromRow, Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct Transaction {
    pub id: Uuid,
    pub sender_address: String,
    pub receiver_address: String,
    pub from_token_address: String,
    pub to_token_address: String,
    pub origin_network: Option<Uuid>,
    pub destin_network: Option<Uuid>,
    pub transfer_amount: i64,
    pub bridge_fee: i64,
    pub tx_status: Option<Uuid>,
    pub origin_tx_hash: Option<String>,
    pub destin_tx_hash: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct RequestInsertTx {
    pub sender_address: String,
    pub receiver_address: String,
    pub from_token_address: String,
    pub to_token_address: String,
    pub origin_network: Option<Uuid>,
    pub destin_network: Option<Uuid>,
    pub transfer_amount: i64,
    pub bridge_fee: i64,
    pub tx_status: Option<Uuid>,
    pub origin_tx_hash: Option<String>,
    pub destin_tx_hash: Option<String>,
    pub created_by: Option<Uuid>,
}

impl Transaction {
    pub async fn get_all_tx(pool: &Pool<Postgres>) -> Result<Vec<Transaction>, sqlx::Error> {
        let transactions = sqlx::query!(
            r#"
            SELECT 
                id,
                sender_address,
                receiver_address,
                from_token_address,
                to_token_address,
                origin_network,
                destin_network,
                transfer_amount,
                bridge_fee,
                tx_status,
                origin_tx_hash,
                destin_tx_hash,
                created_by,
                created_at,
                updated_at
             FROM tbl_transactions
            "#,
        )
        .fetch_all(pool)
        .await?;
        let mut transaction_vec: Vec<Transaction> = Vec::new();
        for transaction in &transactions {
            let new_transaction = Transaction {
                id: transaction.id,
                sender_address: transaction.sender_address.clone(),
                receiver_address: transaction.receiver_address.clone(),
                from_token_address: transaction.from_token_address.clone(),
                to_token_address: transaction.to_token_address.clone(),
                origin_network: transaction.origin_network,
                destin_network: transaction.destin_network,
                transfer_amount: transaction.transfer_amount.unwrap_or(0),
                bridge_fee: transaction.bridge_fee.unwrap_or(0),
                tx_status: transaction.tx_status,
                origin_tx_hash: transaction.origin_tx_hash.clone(),
                destin_tx_hash: transaction.destin_tx_hash.clone(),
                created_by: transaction.created_by,
                created_at: match transaction.created_at {
                    Some(date) => date,
                    None => Utc::now(),
                },
                updated_at: match transaction.updated_at {
                    Some(date) => date,
                    None => Utc::now(),
                },
            };
            transaction_vec.push(new_transaction);
        }
        Ok(transaction_vec)
    }

    pub async fn get_transaction(pool: &Pool<Postgres>, id: Uuid) -> Result<Self, sqlx::Error> {
        let transaction = sqlx::query!(
            r#"
                SELECT 
                    id,
                    sender_address,
                    receiver_address,
                    from_token_address,
                    to_token_address,
                    origin_network,
                    destin_network,
                    transfer_amount,
                    bridge_fee,
                    tx_status,
                    origin_tx_hash,
                    destin_tx_hash,
                    created_by,
                    created_at,
                    updated_at
                FROM tbl_transactions WHERE id = $1
                "#,
            id
        )
        .fetch_one(pool)
        .await?;

        let transaction_response = Self {
            id: transaction.id,
            sender_address: transaction.sender_address.clone(),
            receiver_address: transaction.receiver_address.clone(),
            from_token_address: transaction.from_token_address.clone(),
            to_token_address: transaction.to_token_address.clone(),
            origin_network: transaction.origin_network,
            destin_network: transaction.destin_network,
            transfer_amount: transaction.transfer_amount.unwrap_or(0),
            bridge_fee: transaction.bridge_fee.unwrap_or(0),
            tx_status: transaction.tx_status,
            origin_tx_hash: transaction.origin_tx_hash.clone(),
            destin_tx_hash: transaction.destin_tx_hash.clone(),
            created_by: transaction.created_by,
            created_at: match transaction.created_at {
                Some(date) => date,
                None => Utc::now(),
            },
            updated_at: match transaction.updated_at {
                Some(date) => date,
                None => Utc::now(),
            },
        };
        Ok(transaction_response)
    }

    pub async fn create(pool: &Pool<Postgres>, tx: RequestInsertTx) -> Result<Self, sqlx::Error> {
        let id: Uuid = sqlx::query!(
            "INSERT INTO tbl_transactions (sender_address, receiver_address, from_token_address, to_token_address, origin_network, destin_network, transfer_amount, bridge_fee, tx_status, origin_tx_hash, destin_tx_hash, created_by) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING id",
            tx.sender_address,
            tx.receiver_address,
            tx.from_token_address,
            tx.to_token_address,
            tx.origin_network,
            tx.destin_network,
            tx.transfer_amount,
            tx.bridge_fee,
            tx.tx_status,
            tx.origin_tx_hash,
            tx.destin_tx_hash,
            tx.created_by,
        )
        .fetch_one(pool)
        .await?
        .id;
        Ok(Transaction {
            id,
            sender_address: tx.sender_address.to_owned(),
            receiver_address: tx.receiver_address.to_owned(),
            from_token_address: tx.from_token_address.to_owned(),
            to_token_address: tx.to_token_address.to_owned(),
            origin_network: tx.origin_network,
            destin_network: tx.destin_network,
            transfer_amount: tx.transfer_amount,
            bridge_fee: tx.bridge_fee,
            tx_status: tx.tx_status,
            origin_tx_hash: None,
            destin_tx_hash: None,
            created_by: tx.created_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub async fn update_status(
        pool: &Pool<Postgres>,
        id: Uuid,
        tx_status: Uuid,
    ) -> Result<PgQueryResult, sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE tbl_transactions SET tx_status = $1, updated_at = NOW() WHERE id = $2",
            tx_status,
            id
        )
        .execute(pool)
        .await?;
        Ok(result)
    }

    pub async fn update_tx_hash(
        pool: &Pool<Postgres>,
        id: Uuid,
        origin_tx_hash: Option<String>,
        destin_tx_hash: Option<String>,
    ) -> Result<PgQueryResult, sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE tbl_transactions SET origin_tx_hash = $1, destin_tx_hash = $2, updated_at = NOW() WHERE id = $3",
            origin_tx_hash,
            destin_tx_hash,
            id
        )
        .execute(pool)
        .await?;
        Ok(result)
    }

    pub async fn get_tx_status(
        pool: &Pool<Postgres>,
        id: Uuid,
    ) -> Result<Option<Uuid>, sqlx::Error> {
        let transaction = sqlx::query!(
            r#"
                SELECT 
                    id,
                    tx_status,
                    created_at,
                    updated_at
                FROM tbl_transactions WHERE id = $1
                "#,
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(transaction.tx_status)
    }
}
