use crate::{
    handlers::network_handler::{get_all_networks, get_network_by_id},
    AppState,
};
use axum::{routing::get, Router};
use std::sync::Arc;

pub fn network_routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/networks", get(get_all_networks))
        .route("/network/:id", get(get_network_by_id))
        .with_state(app_state)
}
