use std::str::FromStr;

use anyhow::Error;
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use web3::types::{Address, U256, U64, H256, BlockId};
use web3::types::CallRequest;
use crate::database::model::network::Network;

// static MINIMUN_BLOCK_CONFIRMATION: U64 = U64::from(2);

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

pub async fn get_current_block(pool: &Pool<Postgres>, id: Uuid,) -> Result<U64, Error> {
    let network_rpc = Network::get_network_by_id(pool, id).await?;
    let transport = web3::transports::Http::new(&network_rpc.network_rpc).unwrap();

    let web3 = web3::Web3::new(transport);
    let current_block = web3.eth().block_number().await?;

    Ok(current_block)
}

pub async fn validate_confirmed_block(pool: &Pool<Postgres>, id: Uuid, hash: String) -> Result<bool, Error> {
    let network_rpc = Network::get_network_by_id(pool, id).await?;
    let transport = web3::transports::Http::new(&network_rpc.network_rpc).unwrap();

    let web3 = web3::Web3::new(transport);

    let current_block = get_current_block(pool, id).await?;
    println!("Current Block: {:#?}", current_block);

    //decode the hash to U256
    let hash_trim = hash.trim_start_matches("0x");
    let hash_as_h256 = H256::from_slice(&hex::decode(hash_trim).unwrap());

    let tx_receipt = web3.eth().transaction_receipt(hash_as_h256).await?;

    match tx_receipt {
        Some(tx) => {
            //calculate confirmation block
            let block_hash = BlockId::Hash(tx.block_hash.unwrap_or_default());
            let eth_block = match web3.eth().block_with_txs(block_hash).await? {
                Some(block) => {
                    match block.number {
                        Some(block_num) => block_num,
                        None => return Err(Error::msg("Eth Block not found"))
                    }
                },
                None => return Err(Error::msg("Eth Block not found"))
            };

            println!("Transaction Block: {}", &eth_block);

            let block_confirmation = current_block - eth_block;
            println!("Block Confirmation: {}", &block_confirmation);
            // let (_, flag) = block_confirmation.overflowing_add(U64::from(2));

            if block_confirmation.is_zero() {
                return Err(Error::msg("Block confirmation less than 2, failed"))
            } else {
                match block_confirmation.checked_sub(U64::from(2)) {
                    Some(block_num) => {
                        println!("Number of Confirmation Blocks: {}", &block_num);
                        return Ok(true)
                    },
                    None => return Err(Error::msg("Minimum block confirmation must greater than 0"))
                }  
            }

        },
        None => {
            return Err(Error::msg("Transaction Hash Invalid"))
        }
    }
}