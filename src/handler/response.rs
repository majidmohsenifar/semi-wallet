use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::service::user_coin::service::UserCoin;
use crate::service::auth::service::LoginResult;
use crate::service::auth::service::RegisterResult;
use crate::service::coin::service::Coin;
use crate::service::plan::service::Plan;
use crate::service::order::service::CreateOrderResult;
use crate::service::order::service::OrderDetailResult;


#[derive(Serialize,ToSchema)]
pub struct Empty;

#[derive(Serialize, Deserialize, ToSchema)]
#[aliases(
    ApiResponseCreateUserCoin = ApiResponse<'a,UserCoin>, 
    ApiResponseUserCoinList = ApiResponse<'a,Vec<UserCoin>>,
    ApiResponseLogin = ApiResponse<'a,LoginResult>,
    ApiResponseRegister = ApiResponse<'a,RegisterResult>,
    ApiResponseCoinList = ApiResponse<'a,Vec<Coin>>,
    ApiResponsePlanList = ApiResponse<'a,Vec<Plan>>,
    ApiResponseCreateOrder = ApiResponse<'a,CreateOrderResult>,
    ApiResponseOrderDetail = ApiResponse<'a,OrderDetailResult>,
    ApiResponseEmpty = ApiResponse<'a,Empty>,
)]
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

#[derive(Serialize, Deserialize)]
pub struct ApiError<'a> {
    pub message: &'a str,
}

pub fn error(status_code: StatusCode, message: &str) -> (StatusCode, Json<ApiError<'_>>) {
    (status_code, Json(ApiError { message }))
}
