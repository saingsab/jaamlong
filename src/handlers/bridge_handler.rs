use crate::models::bridge::{Bridge, RequestBridge};
use crate::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

pub async fn get_all_bridge_info(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let networks = Bridge::get_all_bridge_info(&data.db).await;
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

pub async fn get_bridge_by_id(
    State(data): State<Arc<AppState>>,
    Json(payload): Json<RequestBridge>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let bridge = Bridge::get_bridge_info(&data.db, payload.id).await;
    if bridge.is_err() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Something bad happened while fetching all transactions",
        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }
    let data = bridge.unwrap();
    let json_response = serde_json::json!({
        "status": "success",
        "data": data
    });
    Ok(Json(json_response))
}
