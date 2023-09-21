use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ResponseToken {
    pub id: Uuid,
    pub token_address: String,
    pub token_symbol: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestToken {
    pub token_address: String,
    pub token_symbol: String,
    pub asset_type: String,
    pub abi: Option<sqlx::types::JsonValue>,
    pub created_by: Uuid,
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

pub async fn add_token(
    State(data): State<Arc<AppState>>,
    Json(payload): Json<RequestToken>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let insert = sqlx::query!(
        r#"
            INSERT INTO tbl_token_address (token_address, token_symbol, asset_type, created_by)
            VALUES
                ($1, $2, $3,$4);  
        "#,
        payload.token_address,
        payload.token_symbol,
        payload.asset_type,
        payload.created_by,
    )
    .execute(&data.db)
    .await;
    match insert {
        Ok(insert) => {
            println!("Successfully inserted! Result: {:#?}", insert);
        }
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("Error: {}", err)
            });
            return Ok(Json(json_response));
        }
    }
    #[derive(Debug, FromRow, Serialize)]
    pub struct Response {
        id: Uuid,
        token_address: String,
        token_symbol: String,
        asset_type: String,
        created_by: Option<Uuid>,
    }
    let insert_result = sqlx::query_as!(
        Response,
        r#"
            SELECT id, token_address, token_symbol, asset_type, created_by FROM tbl_token_address
        "#
    )
    .fetch_one(&data.db)
    .await;
    match insert_result {
        Ok(result) => {
            let json_response = serde_json::json!({
                "status": "success",
                "data": result
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
