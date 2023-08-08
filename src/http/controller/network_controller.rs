use std::sync::Arc;
use axum::{ extract::State, Json, http::StatusCode, response::IntoResponse };
use crate::database::model::network::Network;
use crate::AppState;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

pub async fn get_all_networks(State(data): State<Arc<AppState>>) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let networks = Network::get_all_networks(&data.db).await;

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