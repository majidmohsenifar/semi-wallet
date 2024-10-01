use axum::{http::StatusCode, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<'a, T: Serialize> {
    pub data: Option<T>,
    pub message: &'a str,
}

pub fn success<T: Serialize>(data: T, message: &str) -> (StatusCode, Json<ApiResponse<T>>) {
    (
        StatusCode::OK,
        Json(ApiResponse {
            data: Some(data),
            message,
        }),
    )
}

#[derive(Serialize)]
pub struct ApiError<'a> {
    pub message: &'a str,
}

pub fn error(status_code: StatusCode, message: &str) -> (StatusCode, Json<ApiError<'_>>) {
    (status_code, Json(ApiError { message }))
}
