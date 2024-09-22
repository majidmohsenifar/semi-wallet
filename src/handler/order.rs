use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{
    service::order::{
        error::OrderError,
        service::{CreateOrderParams, OrderDetailParams},
    },
    SharedState,
};

use super::response::{self};

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
        Err(err) => match err {
            OrderError::NotFound => response::error(StatusCode::NOT_FOUND, "").into_response(),
            OrderError::PlanNotFound => response::error(StatusCode::NOT_FOUND, "").into_response(),
            OrderError::InvalidPaymentProvider => {
                response::error(StatusCode::BAD_REQUEST, "").into_response()
            }
            OrderError::Unknown => {
                response::error(StatusCode::INTERNAL_SERVER_ERROR, "").into_response()
            }
        },
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
        Err(err) => match err {
            OrderError::NotFound => response::error(StatusCode::NOT_FOUND, "").into_response(),
            OrderError::PlanNotFound => response::error(StatusCode::NOT_FOUND, "").into_response(),
            OrderError::InvalidPaymentProvider => {
                response::error(StatusCode::BAD_REQUEST, "").into_response()
            }
            OrderError::Unknown => {
                response::error(StatusCode::INTERNAL_SERVER_ERROR, "").into_response()
            }
        },
    }
}
