use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgQueryResult, FromRow, Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct TransactionStatus {
    pub id: Uuid,
    pub status_name: String,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct RequestInsertTxStatus {
    pub status_name: String,
    pub created_by: Option<Uuid>,
}

impl TransactionStatus {
    pub async fn get_all_tx_status(
        pool: &Pool<Postgres>,
    ) -> Result<Vec<TransactionStatus>, sqlx::Error> {
        let transactions = sqlx::query!(
            r#"
            SELECT 
                id,
                status_name,
                created_by,
                created_at,
                updated_at
             FROM tbl_status
            "#,
        )
        .fetch_all(pool)
        .await?;
        let mut transaction_vec: Vec<Self> = Vec::new();
        for transaction in &transactions {
            let new_transaction = Self {
                id: transaction.id,
                status_name: transaction.status_name.clone(),
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

    pub async fn get_transaction_status(
        pool: &Pool<Postgres>,
        id: Uuid,
    ) -> Result<Self, sqlx::Error> {
        let transaction = sqlx::query!(
            r#"
                SELECT 
                    id,
                    status_name,
                    created_by,
                    created_at,
                    updated_at
                FROM tbl_status WHERE id = $1
                "#,
            id
        )
        .fetch_one(pool)
        .await?;

        let transaction_status_response = Self {
            id: transaction.id,
            status_name: transaction.status_name,
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
        Ok(transaction_status_response)
    }

    pub async fn create(
        pool: &Pool<Postgres>,
        tx_status: RequestInsertTxStatus,
    ) -> Result<Self, sqlx::Error> {
        let id: Uuid = sqlx::query!(
            "INSERT INTO tbl_status (status_name, created_by) VALUES ($1, $2) RETURNING id",
            tx_status.status_name,
            tx_status.created_by,
        )
        .fetch_one(pool)
        .await?
        .id;
        Ok(TransactionStatus {
            id,
            status_name: tx_status.status_name.clone(),
            created_by: tx_status.created_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub async fn update_status(
        pool: &Pool<Postgres>,
        id: Uuid,
        status_name: String,
    ) -> Result<PgQueryResult, sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE tbl_status SET status_name = $1, updated_at = NOW() WHERE id = $2",
            status_name,
            id
        )
        .execute(pool)
        .await?;
        Ok(result)
    }
}
