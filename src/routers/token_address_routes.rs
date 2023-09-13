use crate::handlers::token_address_handler::{
    get_all_token_addresses, get_erc20_token_uuid, get_native_token_uuid,
};
use crate::AppState;
use axum::{routing::get, Router};
use std::sync::Arc;

pub fn token_address_routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/token-addresses", get(get_all_token_addresses))
        .route("/token-uuid-erc20", get(get_erc20_token_uuid))
        .route("/token-uuid-native", get(get_native_token_uuid))
        .with_state(app_state)
}
