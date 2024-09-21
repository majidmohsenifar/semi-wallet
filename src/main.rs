#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_must_use)]

mod client;
mod config;
mod handler;
mod service;

use std::sync::Arc;

use axum::{routing::get, Router};

use client::postgres;
use service::coin::service::Service as CoinService;
use service::order::service::Service as OrderService;
use service::payment::service::Service as PaymentService;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let cfg = config::get_configuration().expect("cannot parse configuration");
    //println!("{:?}", cfg);
    let payment_service = PaymentService::new();
    let db_pool = postgres::new_pg_pool(&cfg.db.dsn).await;
    let order_service = OrderService::new(db_pool, payment_service);
    let coin_service = CoinService::new();
    let app_state = AppState {
        order_service,
        coin_service,
    };
    let shared_state = Arc::new(RwLock::new(app_state));
    let app = Router::new()
        .route("/api/v1/order/detail", get(handler::order::order_detail))
        //.route("/api/v1/order/detail", get(handler::order::order_detail_2))
        .with_state(shared_state);
    let listener = tokio::net::TcpListener::bind(cfg.server.address)
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

type SharedState = Arc<RwLock<AppState>>;

struct AppState {
    order_service: OrderService,
    coin_service: CoinService,
}
