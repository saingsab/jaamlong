use crate::database::model::transaction::Transaction;
use chrono::Utc;
use sqlx::{ Pool, Postgres, postgres::PgQueryResult};
use uuid::Uuid;


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
        sender_address: String,
        receiver_address: String,
        from_token_address: String,
        to_token_address: String,
        origin_network: Option<Uuid>,
        destin_network: Option<Uuid>,
        asset_type: Option<Uuid>,
        transfer_amount: i64,
        bridge_fee: i64,
        tx_status: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> Result<Transaction, sqlx::Error> {
        // let mut pool = pool.acquire().await?;
        let id: Uuid = sqlx::query!(
            "INSERT INTO tbl_transactions (sender_address, receiver_address, from_token_address, to_token_address, origin_network, destin_network, asset_type, transfer_amount, bridge_fee, tx_status, created_by) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING id",
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
        )
        .fetch_one(pool)
        .await?
        .id;

        Ok(Transaction {
            id,
            sender_address: sender_address.to_owned(),
            receiver_address: receiver_address.to_owned(),
            from_token_address: from_token_address.to_owned(),
            to_token_address: to_token_address.to_owned(),
            origin_network,
            destin_network,
            asset_type,
            transfer_amount,
            bridge_fee,
            tx_status,
            created_by,
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