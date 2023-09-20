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

#[derive(Serialize)]
struct NetworkIdResponse {
    id: Uuid,
    network_name: String,
}

pub async fn get_network_by_id(
    State(data): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let network = sqlx::query_as!(
        NetworkIdResponse,
        r#"
            SELECT id, network_name from tbl_networks where id = $1
        "#,
        id
    )
    .fetch_one(&data.db)
    .await;
    match network {
        Ok(network) => {
            let json_response = serde_json::json!({
                "status": "success",
                "data": network
            });
            Ok(Json(json_response))
        }
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("{}", err)
            });
            Ok(Json(json_response))
        }
    }
}

pub async fn get_all_networks(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let all_networks = sqlx::query_as!(
        NetworkIdResponse,
        r#"
            SELECT id, network_name from tbl_networks
        "#
    )
    .fetch_all(&data.db)
    .await;
    match all_networks {
        Ok(networks) => {
            let json_response = serde_json::json!({
                "status": "success",
                "data": networks
            });
            Ok(Json(json_response))
        }
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "fail",
                "data": format!("{}", err)
            });
            Ok(Json(json_response))
        }
    }
}
