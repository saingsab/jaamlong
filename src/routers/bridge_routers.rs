use crate::{
    handlers::bridge_handler::{get_all_bridge_info, get_bridge_by_id},
    AppState,
};
use axum::{routing::get, Router};
use std::sync::Arc;

pub fn bridge_routers(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/bridges", get(get_all_bridge_info))
        .route("/bridge", get(get_bridge_by_id))
        .with_state(app_state)
}
