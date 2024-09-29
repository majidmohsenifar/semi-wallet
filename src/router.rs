use crate::{handler, SharedState};
use axum::{
    routing::{get, post},
    Router,
};

pub async fn get_router(shared_state: SharedState) -> Router {
    let order_routes = Router::new()
        .route("/create", post(handler::order::create_order))
        .route("/detail", get(handler::order::order_detail));

    let coin_routes = Router::new().route("/", get(handler::coin::coins_list));

    let api_routes = Router::new()
        .nest("/orders", order_routes)
        .nest("/coins", coin_routes);

    Router::new()
        .nest("/api/v1", api_routes)
        .with_state(shared_state)
}
