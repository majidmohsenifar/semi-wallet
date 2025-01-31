use axum::{extract::State, response::IntoResponse};

use crate::http_server::SharedState;

use super::response;

#[utoipa::path(
        get,
        path = "/api/v1/coins",
        responses(
            (status = OK, description = "", body = ApiResponseCoinsList),
            (status = INTERNAL_SERVER_ERROR, description = "something went wrong in server")
        )
)]
pub async fn coins_list(State(state): State<SharedState>) -> impl IntoResponse {
    let state = state.read().await;
    let res = state.coin_service.coins_list().await;
    match res {
        Ok(res) => response::success(res, "").into_response(),
        Err(err) => err.into_response(),
    }
}
