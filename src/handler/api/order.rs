use axum::{
    extract::{Query, Request, State},
    http::StatusCode,
    response::IntoResponse,
    Extension,
};

use crate::{
    http_server::SharedState,
    repository::models::User,
    service::order::service::{CreateOrderParams, GetUserOrdersListParams, OrderDetailParams},
};

use super::response;
use validator::Validate;

//use axum_macros::debug_handler;

//#[debug_handler]

#[utoipa::path(
        post,
        path = "/api/v1/orders/create",
        responses(
            (status = OK, description = "", body = ApiResponseCreateOrder),
            (status = INTERNAL_SERVER_ERROR, description = "something went wrong in server"),
            (status = BAD_REQUEST, description = "plan not found"),
            (status = BAD_REQUEST, description = "invalid payment provider")
        ),
        request_body = CreateOrderParams,
        security(
            ("api_jwt_token" = [])
        )
)]
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
            let mut s = e.to_string();
            if s.contains(" at") {
                let parts: Vec<&str> = s.split(" at").collect();
                s = parts[0].to_string();
            }
            return response::error(StatusCode::BAD_REQUEST, &s).into_response();
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
            (status = OK, description = "", body = ApiResponseOrderDetailResult),
            (status = NOT_FOUND, description = "order not found"),
            (status = INTERNAL_SERVER_ERROR, description = "something went wrong in server")
        ),
        params(
            ("id" = u64, Query, description = "order id"),
        ),
        security(
            ("api_jwt_token" = [])
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
        .get_order_detail(user, OrderDetailParams { id: params.id })
        .await;

    match res {
        Ok(res) => response::success(res, "").into_response(),
        Err(err) => err.into_response(),
    }
}

#[utoipa::path(
        get,
        path = "/api/v1/orders",
        responses(
            (status = OK, description = "", body = ApiResponseUserOrdersList),
            (status = INTERNAL_SERVER_ERROR, description = "something went wrong in server")
        ),
        params(
            GetUserOrdersListParams,
        ),
        security(
            ("api_jwt_token" = [])
        )
)]
pub async fn user_orders_list(
    State(state): State<SharedState>,
    Extension(user): Extension<User>,
    req: Request,
) -> impl IntoResponse {
    let params: Query<GetUserOrdersListParams> = match Query::try_from_uri(req.uri()) {
        Ok(p) => p,
        Err(_) => {
            return response::error(StatusCode::BAD_REQUEST, "params are not correct")
                .into_response();
        }
    };
    //if  params.page.is_none() || params.page.unwar

    let state = state.read().await;
    let res = state
        .order_service
        .get_user_orders_list(user, params.0)
        .await;

    match res {
        Ok(res) => response::success(res, "").into_response(),
        Err(err) => err.into_response(),
    }
}
