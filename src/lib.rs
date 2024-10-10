//#![allow(dead_code)]
//#![allow(unused_variables)]
//#![allow(unused_must_use)]

pub mod client;
pub mod config;
pub mod handler;
pub mod http_server;
pub mod middleware;
pub mod repository;
pub mod service;
pub mod telemetry;

use crate::service::auth::service::Service as AuthService;
use crate::service::coin::service::Service as CoinService;
use crate::service::order::service::Service as OrderService;
use crate::service::plan::service::Service as PlanService;
use crate::service::user_coin::service::Service as UserCoinService;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedState = Arc<RwLock<AppState>>;

pub struct AppState {
    pub order_service: OrderService,
    pub coin_service: CoinService,
    pub plan_service: PlanService,
    pub auth_service: AuthService,
    pub user_coin_service: UserCoinService,
}
