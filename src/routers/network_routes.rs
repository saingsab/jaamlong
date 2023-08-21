use std::sync::Arc;
use axum::{
    routing::get,
    Router,
};

use crate::{
    handlers::network_handler::{get_all_networks, get_network_by_id, }, 
    handlers::AppState,
};

pub fn network_routes(app_state: Arc<AppState>) -> Router {
    Router::new() 
        .route("/all_networks", get(get_all_networks))
        .route("/network", get(get_network_by_id))
        .with_state(app_state)
}