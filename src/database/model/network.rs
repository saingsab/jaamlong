use sqlx::{FromRow, Pool, Postgres};
use uuid::Uuid;



#[derive(Debug, FromRow)]
pub struct Network {
    pub id: Uuid,
    pub network_name: String,
    pub created_by: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl Network {
    pub async fn get_network_by_id(pool: &Pool<Postgres>, id: Uuid) -> Result<String, sqlx::Error> {
        let network = sqlx
        ::query!(r#"SELECT network_name FROM tbl_networks WHERE id = $1"#, id)
        .fetch_one(pool).await?;

        Ok(network.network_name)
    }

}