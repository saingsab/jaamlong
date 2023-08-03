use sqlx::{FromRow, Pool, Postgres, postgres::PgQueryResult};
use uuid::Uuid;



#[derive(Debug, FromRow)]
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
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}


impl Transaction {
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
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
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