use crate::handlers::token_address_handler::get_all_token_addresses;
use crate::AppState;
use axum::{routing::get, Router};
use std::sync::Arc;

pub fn token_address_routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/token_addresses", get(get_all_token_addresses))
        .with_state(app_state)
}
