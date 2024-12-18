use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::http_server::SharedState;
use std::str;

use super::response;

//use axum_macros::debug_handler;

//#[debug_handler]
pub async fn handle_stripe_webhook(
    State(state): State<SharedState>,
    req: Request,
) -> impl IntoResponse {
    let (head, body) = req.into_parts();
    let body = match axum::body::to_bytes(body, usize::MAX).await {
        Err(_) => {
            return response::error(StatusCode::BAD_REQUEST, "invalid request body")
                .into_response();
        }
        Ok(t) => t,
    };

    let signature_header = head
        .headers
        .get("Stripe-Signature")
        .and_then(|header| header.to_str().ok());

    let sig = if let Some(sig) = signature_header {
        sig
    } else {
        return response::error(StatusCode::BAD_REQUEST, "invalid header").into_response();
    };

    let req_body = str::from_utf8(&body);
    let req_body = if let Ok(b) = req_body {
        b
    } else {
        return response::error(StatusCode::BAD_REQUEST, "invalid body").into_response();
    };

    let state = state.read().await;
    let res = state
        .order_service
        .handle_stripe_webhook(sig, req_body)
        .await;
    match res {
        Ok(res) => response::success(res, "").into_response(),
        Err(err) => err.into_response(),
    }
}

#[utoipa::path(
        get,
        path = "/api/v1/payment/providers",
        responses(
            (status = OK, description = "", body = ApiResponsePaymentProvidersList),
            (status = INTERNAL_SERVER_ERROR, description = "something went wrong in server")
        )
)]
pub async fn payment_providers(State(state): State<SharedState>) -> impl IntoResponse {
    let state = state.read().await;
    let res = state.payment_service.get_payment_providers().await;
    response::success(res, "").into_response()
}
