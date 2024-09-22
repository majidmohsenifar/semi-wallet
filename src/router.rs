use crate::{handler, SharedState};
use axum::{
    routing::{get, post},
    Router,
};

pub async fn get_router(shared_state: SharedState) -> Router {
    Router::new()
        .route("/api/v1/order/create", post(handler::order::create_order))
        .route("/api/v1/order/detail", get(handler::order::order_detail))
        .with_state(shared_state)
}
