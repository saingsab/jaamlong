use crate::database::model::transaction::Transaction;
use chrono::Utc;
use sqlx::{ Pool, Postgres, postgres::PgQueryResult};
use uuid::Uuid;

pub struct RequestInsertTx {
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
                asset_type,
                transfer_amount,
                bridge_fee,
                tx_status,
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
            origin_network: transaction.origin_network.clone(),
            destin_network: transaction.destin_network.clone(),
            asset_type: transaction.asset_type.clone(),
            transfer_amount: match transaction.transfer_amount { Some(amount) => amount, None => 0},
            bridge_fee: match transaction.bridge_fee { Some(amount) => amount, None => 0},
            tx_status: transaction.tx_status.clone(),
            created_by: transaction.created_by.clone(),
            created_at: match transaction.created_at { Some(date) => date.clone(), None => Utc::now()},
            updated_at: match transaction.updated_at { Some(date) => date.clone(), None => Utc::now()}
        };
        transaction_vec.push(new_transaction);
    }
    
        Ok(transaction_vec)
    }

    pub async fn create(
        pool: &Pool<Postgres>,
        tx: RequestInsertTx
    ) -> Result<Transaction, sqlx::Error> {
        // let mut pool = pool.acquire().await?;
        let id: Uuid = sqlx::query!(
            "INSERT INTO tbl_transactions (sender_address, receiver_address, from_token_address, to_token_address, origin_network, destin_network, asset_type, transfer_amount, bridge_fee, tx_status, created_by) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING id",
            tx.sender_address,
            tx.receiver_address,
            tx.from_token_address,
            tx.to_token_address,
            tx.origin_network,
            tx.destin_network,
            tx.asset_type,
            tx.transfer_amount,
            tx.bridge_fee,
            tx.tx_status,
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
            asset_type: tx.asset_type,
            transfer_amount: tx.transfer_amount,
            bridge_fee: tx.bridge_fee,
            tx_status: tx.tx_status,
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


}