use crate::models::{network::Network, token_address::TokenAddress, transaction::Transaction};
use anyhow::Error;
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
    Address, Block, BlockId, BlockNumber, Bytes, CallRequest, TransactionId, TransactionReceipt,
    H160, H256, U256, U64,
};
#[derive(Debug, Clone, Serialize)]
struct TxRequest {
    pub chain_id: String,
    pub to: String,
    pub nonce: String,
    pub value: String,
    pub gas: String,
    pub gas_price: String,
}
#[derive(Debug, Serialize)]
struct TxBroadcastRequest {
    network_rpc: String,
    bridge_address: String,
    tx: TxRequest,
    token_address: Option<String>,
    abi: Option<Json<Value>>,
}

#[derive(Debug, Deserialize)]
struct ResponseRaw {
    status: String,
    data: Bytes,
}

pub fn generate_error_response(field_name: &str) -> Result<Value, Error> {
    let json_response = serde_json::json!({
        "status": "failed",
        "data": format!("Validation Failed: {}", field_name)
    });
    Ok(json_response)
}

pub async fn get_gas_price(pool: &Pool<Postgres>, id: Uuid) -> Result<U256, Error> {
    let network_rpc = Network::get_network_by_id(pool, id).await?;
    let transport = match web3::transports::Http::new(&network_rpc.network_rpc) {
        Ok(transport) => transport,
        Err(err) => return Err(Error::msg(format!("Error Initialize Transport: {}", err))),
    };
    let web3 = web3::Web3::new(transport);
    match web3.eth().gas_price().await {
        Ok(gas_price) => Ok(gas_price),
        Err(err) => Err(Error::msg(format!("Error getting gas price: {}", err))),
    }
}

pub async fn get_est_gas_price(
    pool: &Pool<Postgres>,
    id: Uuid,
    call_req: CallRequest,
) -> Result<U256, Error> {
    let network_rpc = Network::get_network_by_id(pool, id).await?;
    let transport = match web3::transports::Http::new(&network_rpc.network_rpc) {
        Ok(transport) => transport,
        Err(err) => return Err(Error::msg(format!("Error Initialize Transport: {}", err))),
    };
    let web3 = web3::Web3::new(transport);
    match web3.eth().estimate_gas(call_req, None).await {
        Ok(gas_price) => Ok(gas_price),
        Err(err) => Err(Error::msg(format!("Error getting est gas price: {}", err))),
    }
}

pub async fn validate_account_balance(
    pool: &Pool<Postgres>,
    id: Uuid,
    address: Address,
    transfer_amount: u128,
    token_id: Option<Uuid>,
    network_fee: u128,
) -> Result<bool, Error> {
    let network_rpc = Network::get_network_by_id(pool, id).await?;
    let transport = match web3::transports::Http::new(&network_rpc.network_rpc) {
        Ok(transport) => transport,
        Err(err) => return Err(Error::msg(format!("Error Initialize Transport: {}", err))),
    };
    let web3 = web3::Web3::new(transport);
    let balance: u128;
    if token_id.is_none() {
        let origin_balance = match web3.eth().balance(address, None).await {
            Ok(balance) => balance,
            Err(err) => return Err(Error::msg(format!("Error: {}", err))),
        };
        balance = origin_balance.as_u128();
    } else {
        let token_id = match token_id {
            Some(t_i) => t_i,
            None => return Err(Error::msg("Token ID Not Found or Invalid")),
        };
        let token_balance = match get_erc20_token(pool, id, token_id, address).await {
            Ok(token) => token,
            Err(err) => return Err(Error::msg(format!("Error: {}", err))),
        };
        balance = token_balance;
    }
    if balance < transfer_amount {
        return Err(Error::msg("Account Balance is not sufficient"));
    }
    if balance > network_fee {
        Ok(true)
    } else {
        Err(Error::msg("Account Balance is not sufficient"))
    }
}

pub async fn get_current_block(pool: &Pool<Postgres>, network_id: Uuid) -> Result<U64, Error> {
    let network_rpc = Network::get_network_by_id(pool, network_id).await?;
    let transport = match web3::transports::Http::new(&network_rpc.network_rpc) {
        Ok(transport) => transport,
        Err(err) => return Err(Error::msg(format!("Error Initialize Transport: {}", err))),
    };
    let web3 = web3::Web3::new(transport);
    match web3.eth().block_number().await {
        Ok(current_block) => Ok(current_block),
        Err(err) => Err(Error::msg(format!("Error getting current block: {}", err))),
    }
}

pub async fn get_current_nonce(
    pool: &Pool<Postgres>,
    id: Uuid,
    address: Address,
) -> Result<U256, Error> {
    let network_rpc = Network::get_network_by_id(pool, id).await?;
    let transport = match web3::transports::Http::new(&network_rpc.network_rpc) {
        Ok(transport) => transport,
        Err(err) => return Err(Error::msg(format!("Error Initialize Transport: {}", err))),
    };
    let web3 = web3::Web3::new(transport);
    match web3.eth().transaction_count(address, None).await {
        Ok(nonce) => Ok(nonce),
        Err(err) => Err(Error::msg(format!("Error getting current nonce: {}", err))),
    }
}

pub async fn get_base_fee(
    pool: &Pool<Postgres>,
    id: Uuid,
) -> Result<Block<web3::types::Transaction>, Error> {
    let network_rpc = Network::get_network_by_id(pool, id).await?;
    let transport = match web3::transports::Http::new(&network_rpc.network_rpc) {
        Ok(transport) => transport,
        Err(err) => return Err(Error::msg(format!("Error Initialize Transport: {}", err))),
    };
    let eth = Eth::new(transport.clone());
    let latest_block = match eth
        .block_with_txs(BlockId::Number(BlockNumber::Latest))
        .await
    {
        Ok(block) => block,
        Err(err) => return Err(Error::msg(format!("Error: {}", err))),
    };
    match latest_block {
        Some(block) => Ok(block),
        None => Err(Error::msg("Latest Block Not Found".to_string())),
    }
}

pub async fn get_chain_id(pool: &Pool<Postgres>, id: Uuid) -> Result<U256, Error> {
    let network_rpc = Network::get_network_by_id(pool, id).await?;
    let transport = match web3::transports::Http::new(&network_rpc.network_rpc) {
        Ok(transport) => transport,
        Err(err) => return Err(Error::msg(format!("Error Initialize Transport: {}", err))),
    };
    let web3 = web3::Web3::new(transport);
    match web3.eth().chain_id().await {
        Ok(chain_id) => Ok(chain_id),
        Err(err) => Err(Error::msg(format!("Error getting chain id: {}", err))),
    }
}

pub async fn get_confirmed_block(
    pool: &Pool<Postgres>,
    network_id: Uuid,
    block_hash: BlockId,
) -> Result<U64, Error> {
    let network_rpc = Network::get_network_by_id(pool, network_id).await?;
    let transport = match web3::transports::Http::new(&network_rpc.network_rpc) {
        Ok(transport) => transport,
        Err(err) => return Err(Error::msg(format!("Error Initialize Transport: {}", err))),
    };
    let web3 = web3::Web3::new(transport);
    let current_block = match get_current_block(pool, network_id).await {
        Ok(current_block) => current_block,
        Err(err) => return Err(Error::msg(format!("Error: {}", err))),
    };
    println!("Current Block: {:#?}", current_block);
    //calculate confirmation block
    let eth_block = match web3.eth().block_with_txs(block_hash).await {
        Ok(block) => match block {
            Some(block) => match block.number {
                Some(block_num) => block_num,
                None => return Err(Error::msg("Eth Block not found")),
            },
            None => return Err(Error::msg("Eth Block not found")),
        },
        Err(err) => return Err(Error::msg(format!("Error Getting Block: {}", err))),
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
    let transport = match web3::transports::Http::new(&network_rpc.network_rpc) {
        Ok(transport) => transport,
        Err(err) => return Err(Error::msg(format!("Error Initialize Transport: {}", err))),
    };
    let web3 = web3::Web3::new(transport);
    let hash_trim = hash.trim_start_matches("0x");
    let decode_hash = match hex::decode(hash_trim) {
        Ok(hash) => hash,
        Err(err) => return Err(Error::msg(format!("Error Decode Hash: {}", err))),
    };
    let hash_as_h256 = H256::from_slice(&decode_hash);
    let tx_receipt = match web3.eth().transaction_receipt(hash_as_h256).await {
        Ok(tx_receipt) => tx_receipt,
        Err(err) => return Err(Error::msg(format!("Error Getting Receipt: {}", err))),
    };
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
    let transport = match web3::transports::Http::new(&network_rpc.network_rpc) {
        Ok(transport) => transport,
        Err(err) => return Err(Error::msg(format!("Error Initialize Transport: {}", err))),
    };
    let web3 = web3::Web3::new(transport);
    let hash_trim = hash.trim_start_matches("0x");
    let decode_hash = match hex::decode(hash_trim) {
        Ok(hash) => hash,
        Err(err) => return Err(Error::msg(format!("Error Decode Hash: {}", err))),
    };
    let hash_as_h256 = H256::from_slice(&decode_hash);
    let transaction_id = TransactionId::Hash(hash_as_h256);
    let tx = match web3.eth().transaction(transaction_id).await {
        Ok(tx_receipt) => tx_receipt,
        Err(err) => return Err(Error::msg(format!("Error Getting Receipt: {}", err))),
    };
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

pub async fn contract(
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
    match web3::contract::Contract::from_json(eth, address, &json_bytes) {
        Ok(contract) => Ok(contract),
        Err(err) => Err(Error::msg(format!("Error Initialize Contract: {}", err))),
    }
}

pub async fn token_converter(
    pool: &Pool<Postgres>,
    network_id: Uuid,
    asset_type: String,
    token_id: Option<Uuid>,
    amount: f64,
) -> Result<u128, Error> {
    let network_rpc = Network::get_network_by_id(pool, network_id).await?;
    let transport = match web3::transports::Http::new(&network_rpc.network_rpc) {
        Ok(transport) => transport,
        Err(err) => return Err(Error::msg(format!("Error Initialize Transport: {}", err))),
    };
    if asset_type == "0" {
        let native_decimal = Network::get_network_by_id(pool, network_id).await?;
        let decimal_factor = (10u128).pow(native_decimal.decimal_value as u32);
        let value = amount * (decimal_factor as f64);
        Ok(value as u128)
    } else if asset_type == "1" {
        if let Some(token_id) = token_id {
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
            let decimal_factor = (10u128).pow(decimal.into());
            let value = amount * (decimal_factor as f64);
            Ok(value as u128)
        } else {
            return Err(Error::msg("Invalid Token ID"));
        }
    } else {
        return Err(Error::msg("Invalid Token Asset Type"));
    }
}

pub async fn get_erc20_token(
    pool: &Pool<Postgres>,
    network_id: Uuid,
    token_id: Uuid,
    address: Address,
) -> Result<u128, Error> {
    let network_rpc = Network::get_network_by_id(pool, network_id).await?;
    let transport = match web3::transports::Http::new(&network_rpc.network_rpc) {
        Ok(transport) => transport,
        Err(err) => return Err(Error::msg(format!("Error Initialize Transport: {}", err))),
    };
    let token = TokenAddress::get_token_address_by_id(pool, token_id).await?;
    let token_address = match Address::from_str(token.token_address.as_str()) {
        Ok(address) => address,
        Err(err) => {
            return Err(Error::msg(format!(
                "Fail converting token address: {}",
                err
            )));
        }
    };
    let abi = TokenAddress::get_token_abi_by_id(pool, token_id).await?;
    let contract = contract(transport, token_address, abi).await?;
    let balance: u128 = match contract
        .query(
            "balanceOf",
            Token::Address(address),
            token_address,
            Options::default(),
            BlockId::Number(BlockNumber::Latest),
        )
        .await
    {
        Ok(balance) => balance,
        Err(e) => {
            return Err(Error::msg(format!("Query Address Balance Failed: {}", e)));
        }
    };
    Ok(balance)
}

pub async fn get_decimal(
    pool: &Pool<Postgres>,
    network_id: Uuid,
    token_id: Uuid,
) -> Result<u16, Error> {
    let network_rpc = Network::get_network_by_id(pool, network_id).await?;
    let transport = match web3::transports::Http::new(&network_rpc.network_rpc) {
        Ok(transport) => transport,
        Err(err) => return Err(Error::msg(format!("Error Initialize Transport: {}", err))),
    };
    let token = TokenAddress::get_token_address_by_id(pool, token_id).await?;
    let token_address = match Address::from_str(token.token_address.as_str()) {
        Ok(address) => address,
        Err(err) => {
            return Err(Error::msg(format!(
                "Fail converting token address: {}",
                err
            )));
        }
    };
    let abi = TokenAddress::get_token_abi_by_id(pool, token_id).await?;
    let contract = contract(transport, token_address, abi).await?;
    let decimal: u16 = match contract
        .query(
            "decimals",
            (),
            token_address,
            Options::default(),
            BlockId::Number(BlockNumber::Latest),
        )
        .await
    {
        Ok(decimal) => decimal,
        Err(err) => {
            return Err(Error::msg(format!("Query Decimal Failed: {}", err)));
        }
    };
    Ok(decimal)
}

pub async fn send_erc20(pool: &Pool<Postgres>, transaction: &Transaction) -> Result<H256, Error> {
    let destination_network = match transaction.destin_network {
        Some(network) => network,
        None => {
            return Err(Error::msg("Destination Network Null".to_string()));
        }
    };
    let network = match Network::get_network_by_id(pool, destination_network).await {
        Ok(network) => network,
        Err(err) => return Err(Error::msg(format!("Error: {}", err))),
    };
    let to_token_address_id = match Uuid::from_str(&transaction.to_token_address) {
        Ok(token_id) => token_id,
        Err(err) => return Err(Error::msg(format!("Error getting token id: {}", err))),
    };
    let token = match TokenAddress::get_token_address_by_id(pool, to_token_address_id).await {
        Ok(token) => token,
        Err(err) => return Err(Error::msg(format!("Error query token: {}", err))),
    };
    let abi = match TokenAddress::get_token_abi_by_id(pool, to_token_address_id).await {
        Ok(abi) => abi,
        Err(err) => return Err(Error::msg(format!("Error: {}", err))),
    };
    let actual_amount = transaction.transfer_amount.clone() - transaction.bridge_fee.clone();
    println!("Transfer Amount actual: {}", actual_amount);
    let chain_id = match get_chain_id(pool, destination_network).await {
        Ok(chain_id) => chain_id,
        Err(err) => return Err(Error::msg(format!("Error: {}", err))),
    };
    let bridge_address = match H160::from_str(&network.bridge_address) {
        Ok(address) => address,
        Err(err) => {
            return Err(Error::msg(format!(
                "Fail to get parse bridge address: {}",
                err
            )))
        }
    };
    let nonce = match get_current_nonce(pool, destination_network, bridge_address).await {
        Ok(nonce) => nonce,
        Err(err) => return Err(Error::msg(format!("Error getting nonce: {}", err))),
    };
    let gas_price = match get_gas_price(pool, destination_network).await {
        Ok(gas) => gas,
        Err(err) => return Err(Error::msg(format!("Error getting gas: {}", err))),
    };
    let call_req = CallRequest::builder().build();
    let gas = match get_est_gas_price(pool, destination_network, call_req).await {
        Ok(gas_price) => gas_price,
        Err(err) => return Err(Error::msg(format!("Error getting gas price: {}", err))),
    };
    let tx_req = TxRequest {
        chain_id: chain_id.to_string(),
        to: transaction.receiver_address.clone(),
        nonce: nonce.to_string(),
        value: actual_amount.to_string(),
        gas: gas.to_string(),
        gas_price: gas_price.to_string(),
    };
    let broadcast_tx_field = TxBroadcastRequest {
        network_rpc: network.network_rpc.clone(),
        bridge_address: network.bridge_address,
        tx: tx_req.clone(),
        token_address: Some(token.token_address),
        abi: Some(abi),
    };
    println!("Broadcast Tx Field: {:?}", tx_req.clone());
    let json_body = match serde_json::to_string(&broadcast_tx_field) {
        Ok(json_body) => json_body,
        Err(err) => return Err(Error::msg(format!("Error parsing body: {:?}", err))),
    };
    let client = reqwest::Client::new();
    let res = client
        .post("http://127.0.0.1:7000/sign-erc20-tx")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(json_body)
        .send()
        .await?;
    let res_data = res.text().await?;
    let response: ResponseRaw = serde_json::from_str(&res_data)?;
    println!("Response: {:#?}", response);
    let rlp = if response.status == "success" {
        response.data
    } else {
        return Err(Error::msg("Raw Tx Not Found"));
    };
    let tx_hash = match broadcast_tx(network.network_rpc.clone(), rlp).await {
        Ok(tx_hash) => tx_hash,
        Err(err) => return Err(Error::msg(format!("Error sending transaction: {}", err))),
    };
    println!("Tx hash: {:#?}", &tx_hash);
    Ok(tx_hash)
}

pub async fn send_raw_tx(pool: &Pool<Postgres>, transaction: &Transaction) -> Result<H256, Error> {
    let destination_network = match transaction.destin_network {
        Some(network) => network,
        None => {
            return Err(Error::msg("Destination Network Null".to_string()));
        }
    };
    let network = match Network::get_network_by_id(pool, destination_network).await {
        Ok(network) => network,
        Err(err) => return Err(Error::msg(format!("Error: {}", err))),
    };
    let actual_amount = transaction.transfer_amount.clone() - transaction.bridge_fee.clone();
    println!("Transfer Amount actual: {}", actual_amount);
    let chain_id = match get_chain_id(pool, destination_network).await {
        Ok(chain_id) => chain_id,
        Err(err) => return Err(Error::msg(format!("Error: {}", err))),
    };
    let bridge_address = match H160::from_str(&network.bridge_address) {
        Ok(address) => address,
        Err(err) => {
            return Err(Error::msg(format!(
                "Fail to get parse bridge address: {}",
                err
            )))
        }
    };
    let nonce = match get_current_nonce(pool, destination_network, bridge_address).await {
        Ok(nonce) => nonce,
        Err(err) => return Err(Error::msg(format!("Error getting nonce: {}", err))),
    };
    let gas_price = match get_gas_price(pool, destination_network).await {
        Ok(gas) => gas,
        Err(err) => return Err(Error::msg(format!("Error getting gas: {}", err))),
    };
    let call_req = CallRequest::builder().build();
    let gas = match get_est_gas_price(pool, destination_network, call_req).await {
        Ok(gas_price) => gas_price,
        Err(err) => return Err(Error::msg(format!("Error getting gas price: {}", err))),
    };
    let tx_req = TxRequest {
        chain_id: chain_id.to_string(),
        to: transaction.receiver_address.clone(),
        nonce: nonce.to_string(),
        value: actual_amount.to_string(),
        gas: gas.to_string(),
        gas_price: gas_price.to_string(),
    };
    let broadcast_tx_field = TxBroadcastRequest {
        network_rpc: network.network_rpc.clone(),
        bridge_address: network.bridge_address,
        tx: tx_req.clone(),
        token_address: None,
        abi: None,
    };
    println!("Broadcast tx field: {:?}", broadcast_tx_field);
    let json_body = match serde_json::to_string(&broadcast_tx_field) {
        Ok(json_body) => json_body,
        Err(err) => return Err(Error::msg(format!("Error parsing body: {:?}", err))),
    };
    let client = reqwest::Client::new();
    let res = client
        .post("http://127.0.0.1:7000/sign-raw-tx")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(json_body)
        .send()
        .await?;
    let res_data = res.text().await?;
    let response: ResponseRaw = serde_json::from_str(&res_data)?;
    println!("Response: {:#?}", response);
    let rlp = if response.status == "success" {
        response.data
    } else {
        return Err(Error::msg("Raw Tx Not Found"));
    };
    let tx_hash = match broadcast_tx(network.network_rpc.clone(), rlp).await {
        Ok(tx_hash) => tx_hash,
        Err(err) => return Err(Error::msg(format!("Error sending transaction: {}", err))),
    };
    println!("Tx hash: {:#?}", &tx_hash);
    Ok(tx_hash)
}

async fn broadcast_tx(network_rpc: String, rlp: Bytes) -> Result<H256, Error> {
    let transport = match web3::transports::Http::new(&network_rpc) {
        Ok(transport) => transport,
        Err(err) => return Err(Error::msg(format!("Error Initialize Transport: {}", err))),
    };
    let web3 = web3::Web3::new(transport.clone());
    println!("RLP: {:#?}", rlp);
    let tx_hash = match web3.eth().send_raw_transaction(rlp).await {
        Ok(tx_hash) => tx_hash,
        Err(err) => return Err(Error::msg(format!("Error broadcast: {}", err))),
    };
    Ok(tx_hash)
}
