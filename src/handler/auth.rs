use axum::{extract::State, response::IntoResponse, Json};

use crate::{
    service::auth::service::{LoginParams, RegisterParams},
    SharedState,
};

use super::response;

pub async fn register(
    State(state): State<SharedState>,
    Json(params): Json<RegisterParams>,
) -> impl IntoResponse {
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
