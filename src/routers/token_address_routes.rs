use crate::handlers::token_address_handler::{get_all_token_addresses, get_token_address_by_id};
use crate::AppState;
use axum::{routing::get, Router};
use std::sync::Arc;

pub fn token_address_routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/token-addresses", get(get_all_token_addresses))
        .route("/token-address/:id", get(get_token_address_by_id))
        .with_state(app_state)
}
