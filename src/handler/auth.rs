use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    service::auth::service::{LoginParams, RegisterParams},
    SharedState,
};

use super::response;
use validator::Validate;

pub async fn register(State(state): State<SharedState>, req: Request) -> impl IntoResponse {
    let body = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
        Err(_) => {
            return response::error(StatusCode::BAD_REQUEST, "invalid request body")
                .into_response();
        }
        Ok(b) => b,
    };
    let params: RegisterParams = match serde_json::from_slice(&body) {
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
        if let Some((field, _)) = e.0.into_iter().next() {
            return response::error(StatusCode::BAD_REQUEST, &format!("{} is not valid", field))
                .into_response();
        }
        return response::error(StatusCode::BAD_REQUEST, "invalid payload").into_response();
    }

    if params.password != params.confirm_password {
        return response::error(
            StatusCode::BAD_REQUEST,
            "confirm_password is not the same as password",
        )
        .into_response();
    }

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

pub async fn login(State(state): State<SharedState>, req: Request) -> impl IntoResponse {
    let body = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
        Err(_) => {
            return response::error(StatusCode::BAD_REQUEST, "invalid request body")
                .into_response();
        }
        Ok(b) => b,
    };
    let params: LoginParams = match serde_json::from_slice(&body) {
        Err(e) => {
            return response::error(StatusCode::BAD_REQUEST, &e.to_string()).into_response();
        }
        Ok(p) => p,
    };

    if let Err(e) = params.validate() {
        return response::error(StatusCode::BAD_REQUEST, &e.to_string()).into_response();
    }

    let state = state.read().await;
    let res = state.auth_service.login(LoginParams { ..params }).await;

    match res {
        Ok(res) => response::success(res, "").into_response(),
        Err(err) => err.into_response(),
    }
}
