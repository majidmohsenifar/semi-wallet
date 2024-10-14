use axum::{extract::State, response::IntoResponse};

use crate::SharedState;

use super::response;

#[utoipa::path(
        get,
        path = "/api/v1/plans",
        responses(
            (status = OK, description = "", body = Vec<Plan>),
            (status = INTERNAL_SERVER_ERROR, description = "something went wrong in server")
        )
)]
pub async fn plans_list(State(state): State<SharedState>) -> impl IntoResponse {
    let state = state.read().await;
    let res = state.plan_service.get_plans_list().await;
    match res {
        Ok(res) => response::success(res, "").into_response(),
        Err(err) => err.into_response(),
    }
}
