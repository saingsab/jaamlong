use std::str::FromStr;
use std::sync::Arc;
use axum::{ extract::State, Json, http::StatusCode, response::IntoResponse };
use web3::types::{U256, Address, H160, CallRequest};
use crate::database::{model::{
    transaction::Transaction,  network::Network, token_address::TokenAddress},
    utils::transaction::RequestInsertTx,
};
use crate::http::{
    utils::transaction_module,
    AppState
};
use uuid::Uuid;
use serde::{Deserialize, Serialize};


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
    sender_address: String,
    receiver_address: String,
    transfer_amount: String,
    gas_limit: String,
    max_priority_fee_per_gas: String,
    max_fee_per_gas: String
}

#[derive(Deserialize, Serialize)]
pub struct TestPayload {
    data: String
}

pub async fn get_all_tx(State(data): State<Arc<AppState>>) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
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

// pub async fn insert_tx(State(data): State<Arc<AppState>>, payload: RequestInsertTx) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
//     let create_new_tx = Transaction::create(
//         &data.db, 
//         payload
//     ).await;

//     if create_new_tx.is_err() {
//         let error_response = serde_json::json!({
//             "status": "fail",
//             "message": "Failed to create new transaction",
//         });
//         return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
//     }
    
//     let json_response = serde_json::json!({
//         "status": "success",
//         "data": create_new_tx.unwrap()
//     });

//     Ok(Json(json_response))
// }

// testing post request
// pub async fn test_insert(State(data): State<Arc<AppState>>, Json(payload): Json<TestPayload>) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
//     println!("Received data: {:?}", &payload.data);
//     let data = &payload.data;

//     let json_response = serde_json::json!({
//         "status": "success",
//         "data": data
//     });

//     Ok(Json(json_response))
// }

pub async fn validate_tx(State(data): State<Arc<AppState>>, Json(payload): Json<RequestedTransaction>) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
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

    //validate account
    match transaction_module::validate_account(&data.db, (payload.origin_network).unwrap(), Address::from_str((payload.sender_address).as_str()).unwrap()).await {
        Ok(network) => {},
        Err(err) => {
            let error_message = format!("Error: {}", err);
            let json_response = serde_json::json!({
                "status": "fail",
                "data": error_message
            });
            return Ok(Json(json_response))
        }
    }

    match transaction_module::validate_account(&data.db, (payload.destin_network).unwrap(), Address::from_str((payload.receiver_address).as_str()).unwrap()).await {
        Ok(network) => {},
        Err(err) => {
            let error_message = format!("Error: {}", err);
            let json_response = serde_json::json!({
                "status": "fail",
                "data": error_message
            });
            return Ok(Json(json_response))
        }
    }
    
    //validate network
    let validated_origin_network = match Network::get_network_by_id(&data.db, payload.origin_network.unwrap()).await {
        Ok(network) => network,
        Err(err) => {
            let error_message = format!("Error retrieving network: {}", err);
            let json_response = serde_json::json!({
                "status": "fail",
                "data": error_message
            });
            return Ok(Json(json_response))
        }
    };

    let validated_destinated_network = match Network::get_network_by_id(&data.db, payload.destin_network.unwrap()).await {
        Ok(network) => network,
        Err(err) => {
            let error_message = format!("Error retrieving network: {}", err);
            let json_response = serde_json::json!({
                "status": "fail",
                "data": error_message
            });
            return Ok(Json(json_response))
        }
    };

    //validate tokenID
    let validated_from_token_id = match TokenAddress::get_token_address_by_id(&data.db, payload.from_token_address).await {
        Ok(network) => network,
        Err(err) => {
            let error_message = format!("Error retrieving network: {}", err);
            let json_response = serde_json::json!({
                "status": "fail",
                "data": error_message
            });
            return Ok(Json(json_response))
        }
    };

    let validated_to_token_id = match TokenAddress::get_token_address_by_id(&data.db, payload.to_token_address).await {
        Ok(network) => network,
        Err(err) => {
            let error_message = format!("Error retrieving network: {}", err);
            let json_response = serde_json::json!({
                "status": "fail",
                "data": error_message
            });
            return Ok(Json(json_response))
        }
    };

    //query gas_price
    let current_gas_price = match transaction_module::get_gas_price(&data.db, (payload.origin_network).unwrap()).await {
        Ok(gas_price) => gas_price,
        Err(err) => {
            let error_message = format!("Error retrieving network: {}", err);
            let json_response = serde_json::json!({
                "status": "fail",
                "data": error_message
            });
            return Ok(Json(json_response))
        } 
    };

    let BRIDGE_FEE: U256 = U256::from(100000);

    let call_req = CallRequest {
        from: Some(H160::from_str((payload.sender_address.clone()).as_str()).unwrap()),
        to: Some(H160::from_str((&payload.receiver_address.clone()).as_str()).unwrap()),
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
    let est_gas_price = match transaction_module::get_est_gas_price(&data.db, (payload.origin_network).unwrap(), call_req).await {
        Ok(gas_price) => gas_price,
            Err(err) => {
                let error_message = format!("Error retrieving network: {}", err);
                let json_response = serde_json::json!({
                    "status": "fail",
                    "data": error_message
                });
                return Ok(Json(json_response))
            }
    };

    let response_tx = ResponseTransaction {
        sender_address: payload.sender_address.clone(),
        receiver_address: payload.receiver_address.clone(),
        transfer_amount: (BRIDGE_FEE + U256::from(payload.transfer_amount)).to_string(),
        gas_limit: est_gas_price.to_string(),
        max_priority_fee_per_gas: (U256::from(0)).to_string(),
        max_fee_per_gas: (BRIDGE_FEE + U256::from(payload.transfer_amount)).to_string()
    };

    let inserted_tx = RequestInsertTx {
        sender_address: payload.sender_address.clone(),
        receiver_address: payload.receiver_address.clone(),
        from_token_address: validated_from_token_id.id.to_string(),
        to_token_address: validated_to_token_id.id.to_string(),
        origin_network: Some(validated_origin_network.id),
        destin_network: Some(validated_destinated_network.id),
        asset_type: payload.asset_type,
        transfer_amount: payload.transfer_amount,
        bridge_fee: BRIDGE_FEE.as_u64() as i64,
        tx_status: Some(Uuid::from_u128(0)),
        created_by: Some(Uuid::new_v4())
    };

    //insert unconfirmed tx to database
    match Transaction::create(
        &data.db,
        inserted_tx
    ).await {
        Ok(tx) => {
            println!("{:#?}", tx);
        },
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("{}", err)
            });
            return Ok(Json(json_response))
        }
    }

    
    // ======== Implement Invoke function to transfer from bridge to receiver ==========

    let json_response = serde_json::json!({
        "status": "success",
        "data": response_tx
    });

    Ok(Json(json_response))
}

