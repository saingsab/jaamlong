use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    handlers::transaction_handler::{ get_all_tx, validate_tx, confirm_tx}, 
    handlers::AppState,
};

pub fn transaction_routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(get_all_tx))
        .route("/validate_tx", post(validate_tx))
        .route("/confirm_tx", post(confirm_tx))
        // .route("/api/notes", get(note_list_handler))
        // .route(
        //     "/api/notes/:id",
        //     get(get_note_handler)
        //         .patch(edit_note_handler)
        //         .delete(delete_note_handler),
        // )
        .with_state(app_state)
}