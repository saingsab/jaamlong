use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ResponseToken {
    pub id: Uuid,
    pub token_address: String,
    pub token_symbol: String,
}

pub async fn get_all_token_addresses(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let all_tokens = sqlx::query_as!(
        ResponseToken,
        r#"
            SELECT id, token_address, token_symbol from tbl_token_address
        "#
    )
    .fetch_all(&data.db)
    .await;
    match all_tokens {
        Ok(tokens) => {
            let json_response = serde_json::json!({
                "status": "success",
                "data": tokens
            });
            Ok(Json(json_response))
        }
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("Error: {}", err)
            });
            Ok(Json(json_response))
        }
    }
}

pub async fn get_token_address_by_id(
    State(data): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let token = sqlx::query_as!(
        ResponseToken,
        r#"SELECT id, token_address, token_symbol FROM tbl_token_address WHERE id = $1"#,
        id
    )
    .fetch_one(&data.db)
    .await;
    match token {
        Ok(token) => {
            let json_response = serde_json::json!({
                "status": "success",
                "data": token
            });
            Ok(Json(json_response))
        }
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("Error: {}", err)
            });
            Ok(Json(json_response))
        }
    }
}
