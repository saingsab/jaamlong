use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    http::controller::{
        transaction_controller::{ get_all_tx, insert_tx, test_insert},
        network_controller::{get_all_networks },
        user::{create_user},

    }
    ,
    AppState,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(get_all_tx))
        // .route("/create_tx", post(insert_tx))
        .route("/test_insert", post(test_insert))
        .route("/all_networks", get(get_all_networks))
        // .route("/api/notes", get(note_list_handler))
        // .route(
        //     "/api/notes/:id",
        //     get(get_note_handler)
        //         .patch(edit_note_handler)
        //         .delete(delete_note_handler),
        // )
        .with_state(app_state)
}