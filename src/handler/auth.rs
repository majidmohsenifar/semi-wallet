use std::convert::TryInto;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{
    service::auth::service::{LoginParams, RegisterParams},
    SharedState,
};

use super::response;

pub async fn register(State(state): State<SharedState>, req: Request) -> impl IntoResponse {
    let body = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
        Err(e) => {
            return response::error(StatusCode::BAD_REQUEST, "invalid request body")
                .into_response();
        }
        Ok(t) => t,
    };
    let params: RegisterParams = match serde_json::from_slice(&body) {
        Err(e) => {
            return response::error(StatusCode::BAD_REQUEST, &e.to_string()).into_response();
        }
        Ok(t) => t,
    };
    let state = state.read().await;
    let res = state
        .auth_service
        .register(RegisterParams { ..params })
        .await;

    match res {
        Ok(res) => response::success(res, "").into_response(),
        Err(err) => err.into_response(),
    }
}

pub async fn login(
    State(state): State<SharedState>,
    Json(params): Json<LoginParams>,
) -> impl IntoResponse {
    let state = state.read().await;
    let res = state.auth_service.login(LoginParams { ..params }).await;

    match res {
        Ok(res) => response::success(res, "").into_response(),
        Err(err) => err.into_response(),
    }
}
