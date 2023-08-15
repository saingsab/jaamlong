use anyhow::Error;
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use web3::types::{Address, U256};
use web3::types::CallRequest;
use crate::database::model::network::Network;


pub async fn get_gas_price(pool: &Pool<Postgres>, id: Uuid) -> Result<U256, Error> {
    let network_rpc = Network::get_network_by_id(pool, id).await?;
    let transport = web3::transports::Http::new(&network_rpc.network_rpc).unwrap();

    let web3 = web3::Web3::new(transport);
    let gas_price = web3.eth().gas_price().await?;
    
    Ok(gas_price)
}

pub async fn get_est_gas_price(pool: &Pool<Postgres>, id: Uuid, call_req: CallRequest) -> Result<U256, Error> {
    let network_rpc = Network::get_network_by_id(pool, id).await?;
    let transport = web3::transports::Http::new(&network_rpc.network_rpc).unwrap();

    let web3 = web3::Web3::new(transport);
    let gas_price = web3.eth().estimate_gas(call_req, None).await?;
    
    Ok(gas_price)
}

pub async fn validate_account(pool: &Pool<Postgres>, id: Uuid, address: Address) -> Result<bool, Error> {
    let network_rpc = Network::get_network_by_id(pool, id).await?;
    let transport = web3::transports::Http::new(&network_rpc.network_rpc).unwrap();

    let web3 = web3::Web3::new(transport);
    let balance = web3.eth().balance(address, None).await?;
    
    if balance.as_u128() as f32 > 0.00 {
        return Ok(true)
    } else {
        return Err(Error::msg("Account Invalid"))
    }
}