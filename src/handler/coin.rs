use axum::{extract::State, response::IntoResponse};

use crate::SharedState;

use super::response;

pub async fn coin_list(State(state): State<SharedState>) -> impl IntoResponse {
    let state = state.read().await;
    let res = state.coin_service.coin_list().await;
    match res {
        Ok(res) => response::success(res, "").into_response(),
        Err(err) => err.into_response(),
    }
}
