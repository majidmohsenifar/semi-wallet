use axum::{
    extract::{Request, State},
    http::{self, StatusCode},
    middleware::Next,
    response::IntoResponse,
};

use crate::{handler::api::response, http_server::SharedState};

pub async fn auth_middleware(
    State(state): State<SharedState>,
    mut req: Request,
    next: Next,
) -> impl IntoResponse {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(bearer_token) = auth_header {
        bearer_token
    } else {
        return response::error(StatusCode::UNAUTHORIZED, "invalid token").into_response();
    };

    let mut parts = auth_header.split(' ');

    let token = if let Some(token) = parts.nth(1) {
        token
    } else {
        return response::error(StatusCode::UNAUTHORIZED, "invalid token").into_response();
    };

    let state = state.read().await;

    let user = state.auth_service.get_user_from_token(token).await;

    let user = match user {
        Err(_e) => {
            return response::error(StatusCode::UNAUTHORIZED, "invalid token").into_response();
        }
        Ok(u) => u,
    };

    req.extensions_mut().insert(user);
    next.run(req).await
}
