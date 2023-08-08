pub mod model;
pub mod utils;

use sqlx::PgPool;
use uuid::Uuid;

use crate::database::model::transaction::Transaction;
use crate::database::model::network::Network;
// use crate::database::utils::transaction;

pub async fn transact(pool: PgPool) -> anyhow::Result<()> {

    //create new transaction
    let new_tx = Transaction::create(
        &pool,
        "0x123456789123456789".to_owned(),
        "0x123456789123456789".to_owned(),
        "0x123456789123".to_owned(),
        "0x998712341234".to_owned(),
        Some(Uuid::parse_str("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8")?),
        Some(Uuid::parse_str("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8")?),
        Some(Uuid::parse_str("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8")?),
        12000,
        50000,
        Some(Uuid::parse_str("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8")?),
        Some(Uuid::parse_str("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8")?)
    ).await?;
    println!("Added new transaction with id {:#?}", new_tx);

    //Update transaction Status
    let update_tx = Transaction::update_status(
        &pool,
        new_tx.id,
        Uuid::parse_str("a9a8a7a6b1b2c1c9d1d2d3d4d5d6d9d8")?
    ).await?;
    println!("Row effected: {:#?}", update_tx);

    //Query Transaction after status is being updated
    let tx_after_updated = sqlx
        ::query!(r#"SELECT * FROM tbl_transactions WHERE id = $1"#, new_tx.id)
        .fetch_one(&pool).await?;

    println!("Tx after updated: {:#?}", tx_after_updated.tx_status);

    let all_networks = sqlx::query!(r#"SELECT * FROM tbl_networks"#).fetch_all(&pool).await?;

    let test_network = Network::get_network_by_id(&pool, all_networks[0].id).await?;
    
    println!("Selected Network: {}", test_network);

    Ok(())
}
