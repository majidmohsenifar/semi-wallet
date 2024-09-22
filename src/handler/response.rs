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

pub fn error(status_code: StatusCode, message: &str) -> (StatusCode, Json<ApiResponse<Vec<i8>>>) {
    (
        status_code,
        Json(ApiResponse {
            data: Some(Vec::new()),
            message,
        }),
    )
}
