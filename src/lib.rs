//#![allow(dead_code)]
//#![allow(unused_variables)]
//#![allow(unused_must_use)]

pub mod client;
pub mod config;
pub mod handler;
pub mod repository;
pub mod router;
pub mod service;

use crate::service::coin::service::Service as CoinService;
use crate::service::order::service::Service as OrderService;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedState = Arc<RwLock<AppState>>;

pub struct AppState {
    pub order_service: OrderService,
    pub coin_service: CoinService,
}
