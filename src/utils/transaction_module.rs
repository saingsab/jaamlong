use crate::models::{
    bridge::Bridge, network::Network, token_address::TokenAddress, transaction::Transaction,
};
use anyhow::Error;
use secp256k1::SecretKey;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Json;
use sqlx::{Pool, Postgres};
use std::str::FromStr;
use uuid::Uuid;
use web3::api::{Eth, Namespace};
use web3::contract::Options;
use web3::ethabi::Token;
use web3::transports::Http;
use web3::types::{
    Address, Block, BlockId, BlockNumber, CallRequest, TransactionId, TransactionParameters,
    TransactionReceipt, H160, H256, U256, U64,
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

pub async fn validate_account_balance(
    pool: &Pool<Postgres>,
    id: Uuid,
    address: Address,
    network_fee: u128,
) -> Result<bool, Error> {
    let network_rpc = Network::get_network_by_id(pool, id).await?;
    let transport = web3::transports::Http::new(&network_rpc.network_rpc).unwrap();
    let web3 = web3::Web3::new(transport);
    let balance = web3.eth().balance(address, None).await?;
    let fee = network_fee as f32;
    if balance.as_u128() as f32 > fee {
        Ok(true)
    } else {
        Err(Error::msg("Account Invalid"))
    }
}

pub async fn get_current_block(pool: &Pool<Postgres>, network_id: Uuid) -> Result<U64, Error> {
    let network_rpc = Network::get_network_by_id(pool, network_id).await?;
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

pub async fn get_confirmed_block(
    pool: &Pool<Postgres>,
    network_id: Uuid,
    block_hash: BlockId,
) -> Result<U64, Error> {
    let network_rpc = Network::get_network_by_id(pool, network_id).await?;
    let transport = web3::transports::Http::new(&network_rpc.network_rpc).unwrap();
    let web3 = web3::Web3::new(transport);
    let current_block = get_current_block(pool, network_id).await?;
    println!("Current Block: {:#?}", current_block);

    //calculate confirmation block
    let eth_block = match web3.eth().block_with_txs(block_hash).await? {
        Some(block) => match block.number {
            Some(block_num) => block_num,
            None => return Err(Error::msg("Eth Block not found")),
        },
        None => return Err(Error::msg("Eth Block not found")),
    };
    let block_confirmation = current_block - eth_block;
    Ok(block_confirmation)
}

pub async fn get_tx_receipt(
    pool: &Pool<Postgres>,
    network_id: Uuid,
    hash: String,
) -> Result<TransactionReceipt, Error> {
    let network_rpc = Network::get_network_by_id(pool, network_id).await?;
    let transport = web3::transports::Http::new(&network_rpc.network_rpc).unwrap();
    let web3 = web3::Web3::new(transport);
    let hash_trim = hash.trim_start_matches("0x");
    let hash_as_h256 =
        H256::from_slice(&hex::decode(hash_trim).expect("Failed to decode the hash"));
    let tx_receipt = web3.eth().transaction_receipt(hash_as_h256).await?;
    match &tx_receipt {
        Some(tx) => Ok(tx.clone()),
        None => Err(Error::msg("Transaction Receipt Not Found")),
    }
}

pub async fn get_tx(
    pool: &Pool<Postgres>,
    network_id: Uuid,
    hash: String,
) -> Result<web3::types::Transaction, Error> {
    let network_rpc = Network::get_network_by_id(pool, network_id).await?;
    let transport = web3::transports::Http::new(&network_rpc.network_rpc).unwrap();
    let web3 = web3::Web3::new(transport);
    let hash_trim = hash.trim_start_matches("0x");
    let hash_as_h256 =
        H256::from_slice(&hex::decode(hash_trim).expect("Failed to decode the hash"));
    let transaction_id = TransactionId::Hash(hash_as_h256);
    let tx = web3.eth().transaction(transaction_id).await?;
    match &tx {
        Some(tx) => Ok(tx.clone()),
        None => Err(Error::msg("Transaction Not Found")),
    }
}

pub async fn get_token_supply(
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
    abi_json: Json<Value>,
) -> Result<web3::contract::Contract<Http>, Error> {
    let eth = Eth::new(transport);
    let abi: String = match serde_json::to_string(&abi_json) {
        Ok(abi) => abi,
        Err(err) => {
            return Err(err.into());
        }
    };
    let json_bytes = abi.as_bytes().to_vec();
    let contract = web3::contract::Contract::from_json(eth, address, &json_bytes).unwrap();
    Ok(contract)
}

pub async fn token_converter(
    pool: &Pool<Postgres>,
    network_id: Uuid,
    asset_type: String,
    token_id: Option<Uuid>,
    amount: f64,
) -> Result<u128, Error> {
    let network = Network::get_network_by_id(pool, network_id).await?;
    let transport = web3::transports::Http::new(&network.network_rpc).unwrap();
    if asset_type == "0" {
        let native_decimal = Network::get_network_by_id(pool, network_id).await?;
        let decimal_factor = (10u128).pow(native_decimal.decimal_value as u32);
        let value = amount * (decimal_factor as f64);
        Ok(value as u128)
    } else if asset_type == "1" {
        let token =
            TokenAddress::get_token_address_by_id(pool, token_id.expect("Token ID Not Found"))
                .await?;
        let token_address = match Address::from_str(token.token_address.as_str()) {
            Ok(address) => address,
            Err(err) => {
                return Err(err.into());
            }
        };
        let abi =
            TokenAddress::get_token_abi_by_id(pool, token_id.expect("Token ID Not Found")).await?;
        let contract = contract(transport, token_address, abi).await?;
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
        println!("Decimal factor: {}", decimal_factor);
        let value = amount * (decimal_factor as f64);
        println!("Value: {}", value);
        Ok(value as u128)
    } else {
        return Err(Error::msg("Invalid Token Asset Type"));
    }
}

pub async fn get_decimal(
    pool: &Pool<Postgres>,
    network_id: Uuid,
    token_id: Uuid,
) -> Result<u16, Error> {
    let network = Network::get_network_by_id(pool, network_id).await?;
    let transport = web3::transports::Http::new(&network.network_rpc).unwrap();
    let token = TokenAddress::get_token_address_by_id(pool, token_id).await?;
    let token_address = match Address::from_str(token.token_address.as_str()) {
        Ok(address) => address,
        Err(err) => {
            return Err(err.into());
        }
    };
    let abi = TokenAddress::get_token_abi_by_id(pool, token_id).await?;
    let contract = contract(transport, token_address, abi).await?;
    let decimal: u16 = contract
        .query(
            "decimals",
            (),
            token_address,
            Options::default(),
            BlockId::Number(BlockNumber::Latest),
        )
        .await?;
    Ok(decimal)
}

pub async fn send_raw_transaction(
    pool: &Pool<Postgres>,
    network_id: Uuid,
    transaction: &Transaction,
    p_k: &str,
) -> Result<H256, Error> {
    let network_rpc = Network::get_network_by_id(pool, network_id).await?;
    let transport = web3::transports::Http::new(&network_rpc.network_rpc).unwrap();
    let web3 = web3::Web3::new(transport);
    let bridge_key = dotenvy::var("BRIDGE_KEY").expect("Bridge key is required");
    let bridge = Bridge::get_bridge_info(pool, Uuid::from_str(bridge_key.as_str()).unwrap())
        .await
        .expect("ERROR: Failed to get bridge info");
    let bridge_address = H160::from_str(&bridge.bridge_address.to_owned()).unwrap();
    let nonce = get_current_nonce(pool, network_id, bridge_address).await?;
    let gas = get_gas_price(pool, network_id).await?;
    let call_req = CallRequest::builder().build();
    let gas_price = get_est_gas_price(pool, network_id, call_req).await?;
    println!("Gas: {:?}", &gas);
    println!("Gas estimation: {:?}", &gas_price);
    let tx = TransactionParameters {
        nonce: Some(nonce),
        to: Some(H160::from_str(transaction.receiver_address.as_str()).unwrap()),
        gas: gas_price,
        gas_price: Some(gas),
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

pub async fn send_erc20_token(
    pool: &Pool<Postgres>,
    network_id: Uuid,
    transaction: &Transaction,
    p_k: &str,
) -> Result<H256, Error> {
    let network = Network::get_network_by_id(pool, network_id).await?;
    let transport = web3::transports::Http::new(&network.network_rpc).unwrap();
    let to_token_address_id = Uuid::from_str(transaction.to_token_address.as_str()).unwrap();
    let token = TokenAddress::get_token_address_by_id(pool, to_token_address_id).await?;
    let token_address = match Address::from_str(token.token_address.as_str()) {
        Ok(address) => address,
        Err(err) => {
            return Err(err.into());
        }
    };
    let abi = TokenAddress::get_token_abi_by_id(pool, to_token_address_id).await?;
    let contract = contract(transport, token_address, abi).await?;
    let receiver_address = Address::from_str(transaction.receiver_address.as_str()).unwrap();
    let key = SecretKey::from_str(p_k).unwrap();
    let transfer_amount =
        U256::from(transaction.transfer_amount) - (U256::from(transaction.bridge_fee));
    let send_transaction = match contract
        .signed_call_with_confirmations(
            "transfer",
            (
                Token::Address(receiver_address),
                Token::Uint(transfer_amount),
            ),
            Options::default(),
            2,
            &key,
        )
        .await
    {
        Ok(receipt) => {
            println!("Broadcast Transaciton Receipt: {:#?}", receipt);
            receipt.transaction_hash
        }
        Err(err) => {
            return Err(err.into());
        }
    };
    Ok(send_transaction)
}
