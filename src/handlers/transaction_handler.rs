use crate::models::{
    network::Network,
    token_address::TokenAddress,
    transaction::{RequestInsertTx, Transaction},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::str::FromStr;
use std::sync::Arc;
use web3::types::{Address, CallRequest, H160, U256};

use crate::utils::transaction_module;
use crate::AppState;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct CreateTransaction {
    sender_address: String,
    receiver_address: String,
    from_token_address: String,
    to_token_address: String,
    origin_network: Option<Uuid>,
    destin_network: Option<Uuid>,
    asset_type: Option<Uuid>,
    transfer_amount: i64,
    bridge_fee: i64,
    tx_status: Option<Uuid>,
    created_by: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct RequestedTransaction {
    sender_address: String,
    receiver_address: String,
    from_token_address: Uuid,
    to_token_address: Uuid,
    origin_network: Option<Uuid>,
    destin_network: Option<Uuid>,
    asset_type: Option<Uuid>,
    transfer_amount: i64,
    created_by: Option<Uuid>,
}

#[derive(Serialize)]
pub struct ResponseTransaction {
    id: Uuid,
    sender_address: String,
    receiver_address: String,
    transfer_amount: String,
    gas_limit: String,
    max_priority_fee_per_gas: String,
    max_fee_per_gas: String,
}

#[derive(Deserialize, Serialize)]
pub struct TransactionHash {
    id: Uuid,
    hash: String,
}

pub async fn get_all_tx(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let all_txs = Transaction::get_all_tx(&data.db).await;
    if all_txs.is_err() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Something bad happened while fetching all transactions",
        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }
    let data = all_txs.unwrap();
    let json_response = serde_json::json!({
        "status": "success",
        "data": data
    });
    Ok(Json(json_response))
}

// handle tx validation
pub async fn confirm_tx(
    State(data): State<Arc<AppState>>,
    Json(payload): Json<TransactionHash>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let transaction = match Transaction::get_transaction(&data.db, payload.id).await {
        Ok(tx) => tx,
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("Err: {}", err)
            });
            return Ok(Json(json_response));
        }
    };
    // query network id from transaction id
    let network =
        match Network::get_network_by_id(&data.db, transaction.destin_network.unwrap()).await {
            Ok(network_id) => network_id,
            Err(err) => {
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": format!("Err: {}", err)
                });
                return Ok(Json(json_response));
            }
        };
    match Transaction::get_tx_status(&data.db, payload.id).await {
        Ok(tx_status) => {
            println!(
                "Transaciton ID: {}, \nStatus: {:#?}",
                &payload.id, tx_status
            );
        }
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("Err: {}", err)
            });
            return Ok(Json(json_response));
        }
    }
    match transaction_module::broadcast_tx(&data.db, network.id, payload.id, payload.hash).await {
        Ok(receipt) => {
            let json_response = serde_json::json!({
                "status": "success",
                "data": receipt
            });
            Ok(Json(json_response))
        }
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("Err: {}", err)
            });
            Ok(Json(json_response))
        }
    }
}

pub async fn validate_tx(
    State(data): State<Arc<AppState>>,
    Json(payload): Json<RequestedTransaction>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    //validate request body
    fn generate_error_response(field_name: &str) -> Json<serde_json::Value> {
        let json_response = serde_json::json!({
            "status": "Request Body Failed",
            "data": format!("{} must be provided!", field_name)
        });
        Json(json_response)
    }
    if payload.origin_network.is_none() {
        return Ok(generate_error_response("Origin Network"));
    } else if payload.destin_network.is_none() {
        return Ok(generate_error_response("Destinated Network"));
    } else if payload.asset_type.is_none() {
        return Ok(generate_error_response("Asset Type"));
    } else if payload.created_by.is_none() {
        return Ok(generate_error_response("Creator"));
    }
    // validate sender account
    transaction_module::validate_account(
        &data.db,
        (payload.origin_network).unwrap(),
        Address::from_str((payload.sender_address).as_str()).unwrap(),
    )
    .await
    .unwrap_or_default();
    // validate receiver address
    transaction_module::validate_account(
        &data.db,
        (payload.destin_network).unwrap(),
        Address::from_str((payload.receiver_address).as_str()).unwrap(),
    )
    .await
    .unwrap_or_default();
    //validate origin network
    let validated_origin_network =
        match Network::get_network_by_id(&data.db, payload.origin_network.unwrap()).await {
            Ok(network) => network,
            Err(err) => {
                let error_message = format!("Error retrieving network: {}", err);
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": error_message
                });
                return Ok(Json(json_response));
            }
        };
    //validate destination network
    let validated_destinated_network =
        match Network::get_network_by_id(&data.db, payload.destin_network.unwrap()).await {
            Ok(network) => network,
            Err(err) => {
                let error_message = format!("Error retrieving network: {}", err);
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": error_message
                });
                return Ok(Json(json_response));
            }
        };
    //validate tokenID
    let validated_from_token_id =
        match TokenAddress::get_token_address_by_id(&data.db, payload.from_token_address).await {
            Ok(network) => network,
            Err(err) => {
                let error_message = format!("Error Token Address: {}", err);
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": error_message
                });
                return Ok(Json(json_response));
            }
        };
    let validated_to_token_id =
        match TokenAddress::get_token_address_by_id(&data.db, payload.to_token_address).await {
            Ok(network) => network,
            Err(err) => {
                let error_message = format!("Error Token Address: {}", err);
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": error_message
                });
                return Ok(Json(json_response));
            }
        };
    //query gas_price
    let current_gas_price = match transaction_module::get_gas_price(
        &data.db,
        (payload.origin_network).unwrap(),
    )
    .await
    {
        Ok(gas_price) => gas_price,
        Err(err) => {
            let error_message = format!("Error Gas Price: {}", err);
            let json_response = serde_json::json!({
                "status": "fail",
                "data": error_message
            });
            return Ok(Json(json_response));
        }
    };
    // perform bridge fee calucaltion
    let call_req = CallRequest {
        from: Some(H160::from_str(payload.sender_address.clone().as_str()).unwrap()),
        to: Some(H160::from_str(payload.receiver_address.clone().as_str()).unwrap()),
        gas: None,
        gas_price: Some(current_gas_price),
        value: Some(U256::from(payload.transfer_amount)),
        data: None,
        transaction_type: None,
        access_list: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };
    //estimated_gas_price
    let est_gas_price = match transaction_module::get_est_gas_price(
        &data.db,
        (payload.origin_network).unwrap(),
        call_req,
    )
    .await
    {
        Ok(gas_price) => gas_price,
        Err(err) => {
            let error_message = format!("Error retrieving network: {}", err);
            let json_response = serde_json::json!({
                "status": "fail",
                "data": error_message
            });
            return Ok(Json(json_response));
        }
    };
    // println!("UUid NativeToken: {:#?}", Uuid::new_v5(&Uuid::NAMESPACE_URL, "NativeToken".as_bytes()));
    // println!("UUid ERC20Token: {:#?}", Uuid::new_v5(&Uuid::NAMESPACE_URL, "ERC20Token".as_bytes()));
    let temp_bridge_fee: U256 = U256::from(100000); //mark as constant fee, temporily
    let inserted_tx = RequestInsertTx {
        sender_address: payload.sender_address.clone(),
        receiver_address: payload.receiver_address.clone(),
        from_token_address: validated_from_token_id.id.to_string(),
        to_token_address: validated_to_token_id.id.to_string(),
        origin_network: Some(validated_origin_network.id),
        destin_network: Some(validated_destinated_network.id),
        asset_type: payload.asset_type,
        transfer_amount: payload.transfer_amount,
        bridge_fee: temp_bridge_fee.as_u64() as i64,
        tx_status: Some(Uuid::new_v5(&Uuid::NAMESPACE_URL, "Unconfirmed".as_bytes())),
        origin_tx_hash: None,
        destin_tx_hash: None,
        created_by: payload.created_by,
    };
    //insert unconfirmed tx to database
    let created_tx = match Transaction::create(&data.db, inserted_tx).await {
        Ok(tx) => {
            println!("{:#?}", tx);
            tx
        }
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("{}", err)
            });
            return Ok(Json(json_response));
        }
    };
    let temp_bridge_address = "0xCF6F0d155989B11Ba3882e99c72f609f0C06e086".to_owned();
    let response_tx = ResponseTransaction {
        id: created_tx.id,
        sender_address: payload.sender_address.clone(),
        receiver_address: temp_bridge_address,
        transfer_amount: (temp_bridge_fee + U256::from(payload.transfer_amount)).to_string(),
        gas_limit: est_gas_price.to_string(),
        max_priority_fee_per_gas: est_gas_price.to_string(),
        max_fee_per_gas: (temp_bridge_fee + U256::from(payload.transfer_amount)).to_string(),
    };
    let json_response = serde_json::json!({
        "status": "success",
        "data": response_tx
    });
    Ok(Json(json_response))
}
