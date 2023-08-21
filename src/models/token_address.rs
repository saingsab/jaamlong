use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{ FromRow, Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct TokenAddress {
    pub id: Uuid,
    pub token_address: String,
    pub token_symbol: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


#[derive(Serialize)]
pub struct ResponseTokenAddress {
    pub id: Uuid,
    pub token_address: String,
    pub token_symbol: String
}

impl TokenAddress {
    pub async fn get_token_address_by_id(pool: &Pool<Postgres>, id: Uuid) -> Result<ResponseTokenAddress, sqlx::Error> {
        let network = sqlx
        ::query_as!(ResponseTokenAddress, r#"SELECT id, token_address, token_symbol FROM tbl_token_address WHERE id = $1"#, id)
        .fetch_one(pool).await?;

        Ok(network)
    }

    pub async fn get_all_token_address(pool: &Pool<Postgres>) -> Result<Vec<ResponseTokenAddress>, sqlx::Error> {
        let all_networks = sqlx::query_as!(
            ResponseTokenAddress,
            r#"
                SELECT id, token_address, token_symbol from tbl_token_address
            "#
        ).fetch_all(pool).await?;

        Ok(all_networks)
    }
}