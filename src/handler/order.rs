use axum::{
    extract::{Query, Request, State},
    http::StatusCode,
    response::IntoResponse,
    Extension,
};

use crate::{
    repository::models::User,
    service::order::service::{CreateOrderParams, OrderDetailParams},
    SharedState,
};

use super::response;
use validator::Validate;

//use axum_macros::debug_handler;

//#[debug_handler]
pub async fn create_order(
    State(state): State<SharedState>,
    Extension(user): Extension<User>,
    req: Request,
) -> impl IntoResponse {
    let body = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
        Err(_) => {
            return response::error(StatusCode::BAD_REQUEST, "invalid request body")
                .into_response();
        }
        Ok(b) => b,
    };
    let params: CreateOrderParams = match serde_json::from_slice(&body) {
        Err(e) => {
            return response::error(StatusCode::BAD_REQUEST, &e.to_string()).into_response();
        }
        Ok(p) => p,
    };

    if let Err(e) = params.validate() {
        return response::error(StatusCode::BAD_REQUEST, &e.to_string()).into_response();
    }

    let state = state.read().await;
    let res = state
        .order_service
        .create_order(user, CreateOrderParams { ..params })
        .await;
    match res {
        Ok(res) => response::success(res, "").into_response(),
        Err(err) => err.into_response(),
    }
}

#[utoipa::path(
        get,
        path = "/api/v1/orders/detail",
        responses(
            (status = 200, description = "", body = OrderDetailResult),
            (status = NOT_FOUND, description = "order not found")
        ),
        params(
            ("id" = u64, Query, description = "order id"),
        ),
        security(
            ("token_jwt" = [])
        )
)]
pub async fn order_detail(
    State(state): State<SharedState>,
    Extension(user): Extension<User>,
    req: Request,
) -> impl IntoResponse {
    let params: Query<OrderDetailParams> = match Query::try_from_uri(req.uri()) {
        Ok(p) => p,
        Err(_) => {
            return response::error(StatusCode::BAD_REQUEST, "id is required and must be u64")
                .into_response();
        }
    };
    let state = state.read().await;
    let res = state
        .order_service
        .order_detail(user, OrderDetailParams { id: params.id })
        .await;

    match res {
        Ok(res) => response::success(res, "").into_response(),
        Err(err) => err.into_response(),
    }
}
