use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{
    service::order::{error::OrderError, service::OrderDetailParams},
    SharedState,
};

pub async fn order_detail(
    State(state): State<SharedState>,
    Query(params): Query<OrderDetailParams>,
) -> impl IntoResponse {
    let state = state.read().await;
    let res = state
        .order_service
        .order_detail(OrderDetailParams { id: params.id })
        .await;

    match res {
        Ok(res) => (StatusCode::OK, Json(res)).into_response(),
        Err(err) => match err {
            OrderError::NotFound => (StatusCode::NOT_FOUND, Json("{}")).into_response(),
            OrderError::Unknown => (StatusCode::INTERNAL_SERVER_ERROR, Json("{}")).into_response(),
        },
    }
}
