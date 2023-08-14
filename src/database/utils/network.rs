use uuid::Uuid;
use crate::database::model::network::Network;
use sqlx::{ Pool, Postgres};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseNetwork {
    pub id: Uuid,
    pub network_name: String,
    pub network_rpc: String
}

impl Network {
    pub async fn get_network_by_id(pool: &Pool<Postgres>, id: Uuid) -> Result<ResponseNetwork, sqlx::Error> {
        let network = sqlx
        ::query_as!(ResponseNetwork, r#"SELECT id, network_name, network_rpc FROM tbl_networks WHERE id = $1"#, id)
        .fetch_one(pool).await?;

        Ok(network)
    }

    pub async fn get_all_networks(pool: &Pool<Postgres>) -> Result<Vec<ResponseNetwork>, sqlx::Error> {
        let all_networks = sqlx::query_as!(
            ResponseNetwork,
            r#"
                SELECT id, network_name, network_rpc from tbl_networks
            "#
        ).fetch_all(pool).await?;

        Ok(all_networks)
    }
}