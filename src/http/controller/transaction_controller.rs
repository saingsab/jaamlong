use std::sync::Arc;
use axum::{ extract::State, Json, http::StatusCode, response::IntoResponse };
use crate::database::model::transaction::Transaction;
use crate::AppState;
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

pub async fn insert_tx(State(data): State<Arc<AppState>>, payload: CreateTransaction) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let create_new_tx = Transaction::create(
        &data.db, 
        payload.sender_address, 
        payload.receiver_address, 
        payload.from_token_address, 
        payload.to_token_address,
        payload.origin_network,
        payload.destin_network,
        payload.asset_type,
        payload.transfer_amount,
        payload.bridge_fee,
        payload.tx_status,
        payload.created_by
    ).await;

    if create_new_tx.is_err() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Failed to create new transaction",
        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }
    
    let json_response = serde_json::json!({
        "status": "success",
        "data": create_new_tx.unwrap()
    });

    Ok(Json(json_response))
}

// testing post request
pub async fn test_insert(State(data): State<Arc<AppState>>, Json(payload): Json<TestPayload>) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    println!("Received data: {:?}", &payload.data);
    let data = &payload.data;

    let json_response = serde_json::json!({
        "status": "success",
        "data": data
    });

    Ok(Json(json_response))
}
