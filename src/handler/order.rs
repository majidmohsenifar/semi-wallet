use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};

use crate::{
    service::order::service::{CreateOrderParams, OrderDetailParams},
    SharedState,
};

use super::response;

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
        Ok(res) => response::success(res, "").into_response(),
        Err(err) => err.into_response(),
    }
}

pub async fn create_order(
    State(state): State<SharedState>,
    Json(params): Json<CreateOrderParams>,
) -> impl IntoResponse {
    let state = state.read().await;
    let res = state
        .order_service
        .create_order(CreateOrderParams { ..params })
        .await;
    match res {
        Ok(res) => response::success(res, "").into_response(),
        Err(err) => err.into_response(),
    }
}
