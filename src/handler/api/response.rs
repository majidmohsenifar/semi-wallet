use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::service::auth::service::LoginResult;
use crate::service::auth::service::RegisterResult;
use crate::service::coin::service::Coin;
use crate::service::order::service::CreateOrderResult;
use crate::service::order::service::Order;
use crate::service::order::service::OrderDetailResult;
use crate::service::payment::service::PaymentProvider;
use crate::service::plan::service::Plan;
use crate::service::user_coin::service::UserCoin;

#[derive(Serialize, ToSchema)]
pub struct Empty;

#[derive(Serialize, Deserialize, ToSchema)]
#[aliases(
    ApiResponseCreateUserCoin = ApiResponse<'a,UserCoin>,
    ApiResponseUserCoinsList = ApiResponse<'a,Vec<UserCoin>>,
    ApiResponseLogin = ApiResponse<'a,LoginResult>,
    ApiResponseRegister = ApiResponse<'a,RegisterResult>,
    ApiResponseCoinsList = ApiResponse<'a,Vec<Coin>>,
    ApiResponsePlansList = ApiResponse<'a,Vec<Plan>>,
    ApiResponseCreateOrder = ApiResponse<'a,CreateOrderResult>,
    ApiResponseOrderDetail = ApiResponse<'a,OrderDetailResult>,
    ApiResponseUserOrdersList = ApiResponse<'a,Vec<Order>>,
    ApiResponsePaymentProvidersList = ApiResponse<'a,Vec<PaymentProvider>>,
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
