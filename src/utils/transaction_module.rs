use crate::models::token_address::TokenAddress;
use crate::models::{network::Network, transaction::Transaction};
use anyhow::Error;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::fs::File;
use std::io::Read;
use std::str::FromStr;
use uuid::Uuid;
use web3::api::{Eth, Namespace};
use web3::contract::Options;
use web3::transports::Http;
use web3::types::{
    Address, Block, BlockId, BlockNumber, CallRequest, TransactionParameters, TransactionReceipt,
    H160, H256, U256, U64,
};
#[derive(Deserialize, Serialize)]
pub struct TransactionRequest {
    nonce: Option<U256>,
    to: Address,
    value: U256, // 1 Ether in wei
    gas: Option<U256>,
    gas_price: U256,
    data: Option<U256>,
    chain_id: U256,
}

pub async fn get_gas_price(pool: &Pool<Postgres>, id: Uuid) -> Result<U256, Error> {
    let network_rpc = Network::get_network_by_id(pool, id).await?;
    let transport = web3::transports::Http::new(&network_rpc.network_rpc).unwrap();
    let web3 = web3::Web3::new(transport);
    let gas_price = web3.eth().gas_price().await?;
    Ok(gas_price)
}

pub async fn get_est_gas_price(
    pool: &Pool<Postgres>,
    id: Uuid,
    call_req: CallRequest,
) -> Result<U256, Error> {
    let network_rpc = Network::get_network_by_id(pool, id).await?;
    let transport = web3::transports::Http::new(&network_rpc.network_rpc).unwrap();
    let web3 = web3::Web3::new(transport);
    let gas_price = web3.eth().estimate_gas(call_req, None).await?;
    Ok(gas_price)
}

pub async fn validate_account(
    pool: &Pool<Postgres>,
    id: Uuid,
    address: Address,
) -> Result<bool, Error> {
    let network_rpc = Network::get_network_by_id(pool, id).await?;
    let transport = web3::transports::Http::new(&network_rpc.network_rpc).unwrap();
    let web3 = web3::Web3::new(transport);
    let balance = web3.eth().balance(address, None).await?;
    if balance.as_u128() as f32 > 0.00 {
        println!("Address: {:#?} \nBalance: {}", &address, balance);
        Ok(true)
    } else {
        Err(Error::msg("Account Invalid"))
    }
}

pub async fn get_current_block(pool: &Pool<Postgres>, id: Uuid) -> Result<U64, Error> {
    let network_rpc = Network::get_network_by_id(pool, id).await?;
    let transport = web3::transports::Http::new(&network_rpc.network_rpc).unwrap();
    let web3 = web3::Web3::new(transport);
    let current_block = web3.eth().block_number().await?;
    Ok(current_block)
}

pub async fn get_current_nonce(
    pool: &Pool<Postgres>,
    id: Uuid,
    address: Address,
) -> Result<U256, Error> {
    let network_rpc = Network::get_network_by_id(pool, id).await?;
    let transport = web3::transports::Http::new(&network_rpc.network_rpc).unwrap();
    let web3 = web3::Web3::new(transport);
    let nonce = web3.eth().transaction_count(address, None).await?;
    Ok(nonce)
}

pub async fn get_base_fee(
    pool: &Pool<Postgres>,
    id: Uuid,
) -> Result<Block<web3::types::Transaction>, Error> {
    let network_rpc = Network::get_network_by_id(pool, id).await?;
    let transport = web3::transports::Http::new(&network_rpc.network_rpc).unwrap();
    let eth = Eth::new(transport.clone());
    let latest_block = eth
        .block_with_txs(BlockId::Number(BlockNumber::Latest))
        .await?;
    println!("Latest block: {:#?}", latest_block.clone().unwrap());
    Ok(latest_block.unwrap())
}

pub async fn get_chain_id(pool: &Pool<Postgres>, id: Uuid) -> Result<U256, Error> {
    let network_rpc = Network::get_network_by_id(pool, id).await?;
    let transport = web3::transports::Http::new(&network_rpc.network_rpc).unwrap();
    let web3 = web3::Web3::new(transport);
    let chain_id = web3.eth().chain_id().await?;
    Ok(chain_id)
}

pub async fn validate_confirmed_block(
    pool: &Pool<Postgres>,
    id: Uuid,
    hash: String,
) -> Result<TransactionReceipt, Error> {
    let network_rpc = Network::get_network_by_id(pool, id).await?;
    let transport = web3::transports::Http::new(&network_rpc.network_rpc).unwrap();
    let web3 = web3::Web3::new(transport);
    let current_block = get_current_block(pool, id).await?;
    println!("Current Block: {:#?}", current_block);
    //decode the hash to U256
    let hash_trim = hash.trim_start_matches("0x");
    let hash_as_h256 = H256::from_slice(&hex::decode(hash_trim).unwrap());
    let tx_receipt = web3.eth().transaction_receipt(hash_as_h256).await?;
    match &tx_receipt {
        Some(tx) => {
            //calculate confirmation block
            let block_hash = BlockId::Hash(tx.block_hash.unwrap_or_default());
            let eth_block = match web3.eth().block_with_txs(block_hash).await? {
                Some(block) => match block.number {
                    Some(block_num) => block_num,
                    None => return Err(Error::msg("Eth Block not found")),
                },
                None => return Err(Error::msg("Eth Block not found")),
            };
            let block_confirmation = current_block - eth_block;
            //check if block_confirmation is greater than 2. Negative numbers return None
            match &block_confirmation.checked_sub(U64::from(2)) {
                Some(_block_num) => {
                    println!(
                        "Success, Number of Confirmation Blocks: {}",
                        &block_confirmation
                    );
                    Ok(tx.clone())
                }
                None => Err(Error::msg("Minimum block confirmation must greater than 2")),
            }
        }
        None => Err(Error::msg("Transaction Hash Invalid")),
    }
}

async fn get_token_supply(
    _pool: &Pool<Postgres>,
    _id: Uuid,
    _asset_type: Uuid,
) -> Result<U256, Error> {
    // [1]. If the asset type is the native asset type, then call balance() to check balance inside bridge wallet
    // [2]. If the asset type is the ERC20 asset type, then call total_balance() from smart contract to check balance inside the bridge wallet
    Ok(U256::from(1000))
}

async fn contract(
    transport: Http,
    address: Address,
    abi_path: &str,
) -> Result<web3::contract::Contract<Http>, Error> {
    let eth = Eth::new(transport);
    let abi = get_abi(abi_path).await?;
    let contract = web3::contract::Contract::from_json(eth, address, &abi).unwrap();
    Ok(contract)
}

async fn get_abi(path: &str) -> Result<Vec<u8>, Error> {
    let file_path = path;
    let mut file = File::open(file_path)?;
    // Read the content of the file into a string
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    // Convert the JSON string into a byte slice (Vec<u8>)
    let json_bytes = content.as_bytes().to_vec();
    Ok(json_bytes)
}

pub async fn token_converter(
    pool: &Pool<Postgres>,
    network_id: Uuid,
    token_id: Uuid,
    amount: f64,
) -> Result<u64, Error> {
    let network = Network::get_network_by_id(pool, network_id).await?;
    let token = TokenAddress::get_token_address_by_id(pool, token_id).await?;
    let transport = web3::transports::Http::new(&network.network_rpc).unwrap();
    let path = "src/HSM/NP_ERC20.json";
    let token_address = match Address::from_str(token.token_address.as_str()) {
        Ok(address) => address,
        Err(err) => {
            return Err(err.into());
        }
    };
    let contract = contract(transport, token_address, path).await?;
    let decimal: u16 = contract
        .query(
            "decimals",
            (),
            token_address,
            Options::default(),
            BlockId::Number(BlockNumber::Latest),
        )
        .await?;
    let decimal_factor = (10u128).pow(decimal.into());
    let value = amount * (decimal_factor as f64);
    println!("Value: {}", value);
    Ok(value as u64)
}

pub async fn broadcast_tx(
    pool: &Pool<Postgres>,
    id: Uuid,
    tx_id: Uuid,
    tx_hash: String,
) -> Result<(), Error> {
    match validate_confirmed_block(pool, id, tx_hash.clone()).await {
        Ok(_tx) => {}
        Err(err) => {
            let err_message = format!("Error: {}", err);
            return Err(Error::msg(err_message));
        }
    }
    match Transaction::update_tx_hash(pool, id, Some(tx_hash.clone()), None).await {
        Ok(_tx) => {
            println!("{:#?}", _tx);
        }
        Err(err) => {
            let err_message = format!("Error: {}", err);
            return Err(Error::msg(err_message));
        }
    }
    let transaction = Transaction::get_transaction(pool, tx_id).await?;
    println!("Transaction from db: {:#?}", transaction);
    //validate token amount in the pool
    let total_supply = get_token_supply(pool, id, transaction.asset_type.unwrap()).await?;
    if total_supply.is_zero() {
        return Err(Error::msg("Total Supply is insufficient"));
    }
    #[derive(Debug, Deserialize, Default)]
    struct Signature {
        #[allow(dead_code)]
        ecdsa_sig: EcdsaSig,
    }
    #[derive(Debug, Deserialize, Default)]
    struct EcdsaSig {
        #[allow(dead_code)]
        v: u64,
        #[allow(dead_code)]
        r: Vec<u64>,
        #[allow(dead_code)]
        s: Vec<u64>,
    }
    let file_path = "src/HSM/temp_hsm.json";
    let mut file = File::open(file_path).expect("Failed to open file");
    // Read the content of the file into a string
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Failed to read file");
    let signature: Signature = serde_json::from_str(&content).expect("Failed to parse json");
    println!("Signature: {:#?}", signature);
    if transaction.asset_type == Some(Uuid::new_v5(&Uuid::NAMESPACE_URL, "NativeToken".as_bytes()))
    {
        match send_raw_transaction(pool, id, transaction, "PRIVATE_KEY").await {
            Ok(tx) => {
                println!("Broadcast transaction: {:#?}", tx);
            }
            Err(err) => {
                let err_message = format!("Error: {}", err);
                return Err(Error::msg(err_message));
            }
        };
    } else if transaction.asset_type
        == Some(Uuid::new_v5(&Uuid::NAMESPACE_URL, "ERC20Token".as_bytes()))
    {
        match send_erc20_token(
            pool,
            id,
            transaction,
            "124ce2df311216d9c6f8c417ce2258ef45df6c6e2cb12b40762d1debc8a170e4",
        )
        .await
        {
            Ok(tx) => {
                println!("Result Transaction: {:#?}", tx);
            }
            Err(err) => {
                let err_message = format!("Error: {}", err);
                return Err(Error::msg(err_message));
            }
        }
    }
    Ok(())
}

async fn send_raw_transaction(
    pool: &Pool<Postgres>,
    id: Uuid,
    transaction: Transaction,
    p_k: &str,
) -> Result<H256, Error> {
    let network_rpc = Network::get_network_by_id(pool, id).await?;
    let transport = web3::transports::Http::new(&network_rpc.network_rpc).unwrap();
    let web3 = web3::Web3::new(transport);
    let temp_address = H160::from_str("0xCF6F0d155989B11Ba3882e99c72f609f0C06e086").unwrap();
    let nonce = get_current_nonce(pool, id, temp_address).await?;
    let gas = get_gas_price(pool, id).await?;
    let call_req = CallRequest {
        from: Some(H160::from_str((transaction.sender_address.clone()).as_str()).unwrap()),
        to: Some(H160::from_str(transaction.receiver_address.clone().as_str()).unwrap()),
        gas: None,
        gas_price: Some(gas),
        value: Some(U256::from(transaction.transfer_amount)),
        data: None,
        transaction_type: None,
        access_list: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };
    let gas_price = get_est_gas_price(pool, id, call_req).await?;
    let tx = TransactionParameters {
        nonce: Some(nonce),
        to: Some(H160::from_str(transaction.receiver_address.as_str()).unwrap()),
        gas,
        gas_price: Some(gas_price),
        value: U256::from(transaction.transfer_amount),
        ..Default::default()
    };
    // Sign the transaction
    let signed_tx = web3
        .accounts()
        .sign_transaction(tx, &p_k.parse().unwrap())
        .await?;
    // Send the transaction
    let tx_hash = web3
        .eth()
        .send_raw_transaction(signed_tx.raw_transaction)
        .await?;
    Ok(tx_hash)
}

async fn send_erc20_token(
    _pool: &Pool<Postgres>,
    _id: Uuid,
    _transaction: Transaction,
    _p_k: &str,
) -> Result<H256, Error> {
    todo!();
}
