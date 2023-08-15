use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    http::{controller::{
        transaction_controller::{ get_all_tx, validate_tx},
        network_controller::{get_all_networks, get_network_by_id, },
        token_address_controller::{get_all_token_addresses, get_token_address_by_id,},
        user::{create_user},

    }, utils::transaction_module::validate_account}
    ,
    http::AppState,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(get_all_tx))
        // .route("/create_tx", post(insert_tx))
        // .route("/test_insert", post(test_insert))
        .route("/all_networks", get(get_all_networks))
        .route("/get_network", get(get_network_by_id))
        .route("/get_token_addresses", get(get_all_token_addresses))
        .route("/validate_tx", post(validate_tx))
        // .route("/api/notes", get(note_list_handler))
        // .route(
        //     "/api/notes/:id",
        //     get(get_note_handler)
        //         .patch(edit_note_handler)
        //         .delete(delete_note_handler),
        // )
        .with_state(app_state)
}