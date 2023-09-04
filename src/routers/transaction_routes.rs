use crate::handlers::transaction_handler::{broadcast_tx, get_all_tx, validate_tx};
use crate::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub fn transaction_routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(get_all_tx))
        .route("/validate_tx", post(validate_tx))
        .route("/broadcast_tx", post(broadcast_tx))
        // .route("/api/notes", get(note_list_handler))
        // .route(
        //     "/api/notes/:id",
        //     get(get_note_handler)
        //         .patch(edit_note_handler)
        //         .delete(delete_note_handler),
        // )
        .with_state(app_state)
}
