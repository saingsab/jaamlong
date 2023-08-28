use crate::AppState;
use crate::{
    models::{
        network::Network,
        token_address::TokenAddress,
        transaction::{RequestInsertTx, Transaction},
    },
    utils::transaction_module::{
        broadcast_tx, get_base_fee, get_est_gas_price, get_gas_price, token_converter,
        validate_account,
    },
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;
use web3::types::{Address, CallRequest, H160, U256};

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
    transfer_amount: f64,
    created_by: Option<Uuid>,
}

#[derive(Serialize)]
pub struct ResponseTransaction {
    id: Uuid,
    sender_address: String,
    receiver_address: String,
    transfer_amount: u64,
    gas_limit: String,
    max_priority_fee_per_gas: i64,
    max_fee_per_gas: u64,
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
    match broadcast_tx(&data.db, network.id, payload.id, payload.hash).await {
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
    } else if payload.transfer_amount <= 0.00 {
        let json_response = serde_json::json!({
            "status": "Request Body Failed",
            "data": "Amount must greater than zero",
        });
        return Ok(Json(json_response));
    }
    // validate sender account
    validate_account(
        &data.db,
        (payload.origin_network).unwrap(),
        Address::from_str((payload.sender_address).as_str()).unwrap(),
    )
    .await
    .unwrap_or_default();
    // validate receiver address
    validate_account(
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
                let error_message = format!("Error retrieving origin network: {}", err);
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
                let error_message = format!("Error retrieving destin network: {}", err);
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": error_message
                });
                return Ok(Json(json_response));
            }
        };
    //validate tokenID
    let validated_from_token =
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
    let validated_to_token =
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
    let current_gas_price = match get_gas_price(&data.db, (payload.origin_network).unwrap()).await {
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
    // println!("UUid NativeToken: {:#?}", Uuid::new_v5(&Uuid::NAMESPACE_URL, "NativeToken".as_bytes()));
    // println!("UUid ERC20Token: {:#?}", Uuid::new_v5(&Uuid::NAMESPACE_URL, "ERC20Token".as_bytes()));
    // validate asset type
    if payload.asset_type != Some(Uuid::new_v5(&Uuid::NAMESPACE_URL, "NativeToken".as_bytes()))
        && payload.asset_type != Some(Uuid::new_v5(&Uuid::NAMESPACE_URL, "ERC20Token".as_bytes()))
    {
        let error_message = "Asset type not supported";
        let json_response = serde_json::json!({
            "status": "fail",
            "data": error_message
        });
        return Ok(Json(json_response));
    }
    // perform token conversion
    let transfer_value = match token_converter(
        &data.db,
        validated_origin_network.id,
        payload.asset_type.unwrap(),
        validated_from_token.id,
        payload.transfer_amount,
    )
    .await
    {
        Ok(value) => value,
        Err(err) => {
            let error_message = format!("Error converting token value: {}", err);
            let json_response = serde_json::json!({
                "status": "fail",
                "data": error_message
            });
            return Ok(Json(json_response));
        }
    };
    let call_req = CallRequest {
        from: Some(H160::from_str(payload.sender_address.clone().as_str()).unwrap()),
        to: Some(H160::from_str(payload.receiver_address.clone().as_str()).unwrap()),
        gas: None,
        gas_price: Some(current_gas_price),
        value: Some(U256::from(transfer_value)),
        data: None,
        transaction_type: None,
        access_list: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };
    //estimated_gas_price and balance validation
    let est_gas_price =
        match get_est_gas_price(&data.db, (payload.origin_network).unwrap(), call_req).await {
            Ok(gas_price) => gas_price,
            Err(err) => {
                let error_message = format!("Error retrieving est gas price: {}", err);
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": error_message
                });
                return Ok(Json(json_response));
            }
        };
    // Calculation of the bridge fee as needed
    const TEMP_FEE: f64 = 0.001;
    let temp_bridge_fee = match token_converter(
        &data.db,
        validated_origin_network.id,
        payload.asset_type.unwrap(),
        validated_from_token.id,
        TEMP_FEE,
    )
    .await
    {
        Ok(value) => value,
        Err(err) => {
            let error_message = format!("Error retrieving est gas price: {}", err);
            let json_response = serde_json::json!({
                "status": "fail",
                "data": error_message
            });
            return Ok(Json(json_response));
        }
    }; //mark as constant fee, temporily
    let inserted_tx = RequestInsertTx {
        sender_address: payload.sender_address.clone(),
        receiver_address: payload.receiver_address.clone(),
        from_token_address: validated_from_token.id.to_string(),
        to_token_address: validated_to_token.id.to_string(),
        origin_network: Some(validated_origin_network.id),
        destin_network: Some(validated_destinated_network.id),
        asset_type: payload.asset_type,
        transfer_amount: transfer_value as i64,
        bridge_fee: temp_bridge_fee as i64,
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
    let base_fee = match get_base_fee(&data.db, validated_origin_network.id).await {
        Ok(block) => {
            println!("{:#?}", &block);
            match block.base_fee_per_gas {
                Some(base_gas_fee) => base_gas_fee,
                None => U256::from(0),
            }
        }
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("{}", err)
            });
            return Ok(Json(json_response));
        }
    };
    const TEMP_BRIDGE_ADDRESS: &str = "0xCF6F0d155989B11Ba3882e99c72f609f0C06e086";
    let response_tx = ResponseTransaction {
        id: created_tx.id,
        sender_address: payload.sender_address.clone(),
        receiver_address: TEMP_BRIDGE_ADDRESS.to_string(),
        transfer_amount: (U256::from(temp_bridge_fee) + U256::from(transfer_value)).as_u64(),
        gas_limit: est_gas_price.to_string(),
        max_priority_fee_per_gas: 0,
        max_fee_per_gas: base_fee.as_u64(),
    };
    let json_response = serde_json::json!({
        "status": "success",
        "data": response_tx
    });
    Ok(Json(json_response))
}
