use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgQueryResult, FromRow, Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct Bridge {
    pub id: Uuid,
    pub bridge_address: String,
    pub bridge_fee: f64,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ResponseBridge {
    pub id: Uuid,
    pub bridge_address: String,
    pub bridge_fee: f64,
    pub created_by: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct RequestBridge {
    pub id: Uuid,
    pub bridge_address: String,
    pub bridge_fee: f32,
}

impl Bridge {
    pub async fn get_all_bridge_info(pool: &Pool<Postgres>) -> Result<Vec<Self>, sqlx::Error> {
        let bridges = sqlx::query!(
            r#"
            SELECT 
                id,
                bridge_address,
                bridge_fee,
                created_by,
                created_at,
                updated_at
             FROM tbl_bridge
            "#,
        )
        .fetch_all(pool)
        .await?;
        let mut bridge_vec: Vec<Self> = Vec::new();
        for bridge in &bridges {
            let new_transaction = Self {
                id: bridge.id,
                bridge_address: bridge.bridge_address.clone(),
                bridge_fee: bridge.bridge_fee,
                created_by: bridge.created_by.unwrap(),
                created_at: match bridge.created_at {
                    Some(date) => date,
                    None => Utc::now(),
                },
                updated_at: match bridge.updated_at {
                    Some(date) => date,
                    None => Utc::now(),
                },
            };
            bridge_vec.push(new_transaction);
        }
        Ok(bridge_vec)
    }

    pub async fn get_bridge_info(pool: &Pool<Postgres>, id: Uuid) -> Result<Self, sqlx::Error> {
        let bridges = sqlx::query_as!(
            Self,
            r#"
            SELECT 
                id,
                bridge_address,
                bridge_fee,
                created_by as "created_by!",
                created_at as "created_at!",
                updated_at as "updated_at!"
             FROM tbl_bridge WHERE id = $1
            "#,
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(bridges)
    }

    pub async fn update_bridge_fee(
        pool: &Pool<Postgres>,
        id: Uuid,
        bridge_fee: f64,
    ) -> Result<PgQueryResult, sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE tbl_bridge SET bridge_fee = $1, updated_at = NOW() WHERE id = $2",
            bridge_fee,
            id
        )
        .execute(pool)
        .await?;
        Ok(result)
    }

    pub async fn update_bridge_address(
        pool: &Pool<Postgres>,
        id: Uuid,
        new_address: String,
    ) -> Result<PgQueryResult, sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE tbl_bridge SET bridge_address = $1, updated_at = NOW() WHERE id = $2",
            new_address,
            id
        )
        .execute(pool)
        .await?;
        Ok(result)
    }
}
