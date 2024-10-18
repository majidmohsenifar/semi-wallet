use std::collections::HashMap;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::IntoResponse,
    Extension,
};

use crate::{
    repository::models::User, service::user_coin::service::CreateUserCoinParams, SharedState,
};
use validator::Validate;

use super::response;

#[utoipa::path(
        get,
        path = "/api/v1/user-coins",
        responses(
            (status = OK, description = "", body = ApiResponseUserCoinsList),
            (status = INTERNAL_SERVER_ERROR, description = "something went wrong in server")
        ),
        security(
            ("api_jwt_token" = [])
        )
)]
pub async fn user_coins_list(
    State(state): State<SharedState>,
    Extension(user): Extension<User>,
) -> impl IntoResponse {
    let state = state.read().await;
    let res = state.user_coin_service.get_user_coins_list(user).await;
    match res {
        Ok(res) => response::success(res, "").into_response(),
        Err(err) => err.into_response(),
    }
}

#[utoipa::path(
        post,
        path = "/api/v1/user-coins/create",
        responses(
            (status = OK, description = "", body = ApiResponseCreateUserCoin),
            (status = INTERNAL_SERVER_ERROR, description = "something went wrong in server"),
            (status = BAD_REQUEST, description = "plan not found"),
            (status = BAD_REQUEST, description = "invalid payment provider")
        ),
        request_body = CreateUserCoinParams,
        security(
            ("api_jwt_token" = [])
        )
)]
pub async fn create_user_coin(
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
    let params: CreateUserCoinParams = match serde_json::from_slice(&body) {
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
    let res = state.user_coin_service.create_user_coin(user, params).await;
    match res {
        Ok(res) => response::success(res, "").into_response(),
        Err(err) => err.into_response(),
    }
}

#[utoipa::path(
        delete,
        path = "/api/v1/user-coins/delete",
        responses(
            (status = OK, description = "", body = ApiResponseEmpty),
            (status = INTERNAL_SERVER_ERROR, description = "something went wrong in server"),
            (status = NOT_FOUND, description = "not found"),
        ),
        request_body = CreateUserCoinParams,
        security(
            ("api_jwt_token" = [])
        )
)]
pub async fn delete_user_coin(
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

    let params: HashMap<&str, i64> = match serde_json::from_slice(&body) {
        Err(e) => {
            return response::error(StatusCode::BAD_REQUEST, &e.to_string()).into_response();
        }
        Ok(p) => p,
    };
    let id = match params.get("id") {
        None => {
            return response::error(StatusCode::BAD_REQUEST, "id is required").into_response();
        }
        Some(id) => *id,
    };

    let state = state.read().await;
    let res = state.user_coin_service.delete_user_coin(user, id).await;
    match res {
        Ok(res) => response::success(res, "").into_response(),
        Err(err) => err.into_response(),
    }
}

#[utoipa::path(
        patch,
        path = "/api/v1/user-coins/update-address",
        responses(
            (status = OK, description = "", body = ApiResponseEmpty),
            (status = INTERNAL_SERVER_ERROR, description = "something went wrong in server"),
            (status = NOT_FOUND, description = "not found"),
        ),
        request_body = CreateUserCoinParams,
        security(
            ("api_jwt_token" = [])
        )
)]

pub async fn update_user_coin_address(
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
    let params: HashMap<&str, serde_json::Value> = match serde_json::from_slice(&body) {
        Err(e) => {
            return response::error(StatusCode::BAD_REQUEST, &e.to_string()).into_response();
        }
        Ok(p) => p,
    };
    let id = match params.get("id") {
        None => {
            return response::error(StatusCode::BAD_REQUEST, "id is required").into_response();
        }
        Some(val) => match val.as_i64() {
            None => {
                return response::error(StatusCode::BAD_REQUEST, "id is not i64").into_response();
            }
            Some(id) => id,
        },
    };

    let address = match params.get("address") {
        None => {
            return response::error(StatusCode::BAD_REQUEST, "address is required").into_response();
        }
        Some(val) => match val.as_str() {
            None => {
                return response::error(StatusCode::BAD_REQUEST, "address is not string")
                    .into_response();
            }
            Some(addr) => addr,
        },
    };

    let state = state.read().await;
    let res = state
        .user_coin_service
        .update_user_coin_address(user, id, address)
        .await;
    match res {
        Ok(res) => response::success(res, "").into_response(),
        Err(err) => err.into_response(),
    }
}
