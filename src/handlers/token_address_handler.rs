use crate::models::token_address::TokenAddress;
use crate::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenAddressRequest {
    id: Uuid,
}

pub async fn get_all_token_addresses(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let networks = TokenAddress::get_all_token_address(&data.db).await;
    if networks.is_err() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Something bad happened while fetching all transactions",
        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }
    let data = networks.unwrap();
    let json_response = serde_json::json!({
        "status": "success",
        "data": data
    });
    Ok(Json(json_response))
}

pub async fn get_token_address_by_id(
    State(data): State<Arc<AppState>>,
    Json(payload): Json<TokenAddressRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let networks = TokenAddress::get_token_address_by_id(&data.db, payload.id).await;
    if networks.is_err() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Something bad happened while fetching all transactions",
        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }
    let data = networks.unwrap();
    let json_response = serde_json::json!({
        "status": "success",
        "data": data
    });
    Ok(Json(json_response))
}
