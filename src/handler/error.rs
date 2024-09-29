use axum::{http::StatusCode, response::IntoResponse, Json};

use crate::service::{coin::error::CoinError, order::error::OrderError};

use super::response::ApiError;

impl IntoResponse for OrderError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::PlanNotFound { .. } => StatusCode::BAD_REQUEST,
            Self::InvalidPaymentProvider => StatusCode::BAD_REQUEST,
            Self::Unexpected { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (
            status_code,
            Json(ApiError {
                message: &self.to_string(),
            }),
        )
            .into_response()
    }
}

impl IntoResponse for CoinError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::Unexpected { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (
            status_code,
            Json(ApiError {
                message: &self.to_string(),
            }),
        )
            .into_response()
    }
}
