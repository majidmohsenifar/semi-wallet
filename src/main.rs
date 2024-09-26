use std::sync::Arc;

use semi_wallet::client::postgres;
use semi_wallet::repository::db::Repository;
use semi_wallet::service::order::service::Service as OrderService;
use semi_wallet::service::payment::service::Service as PaymentService;
use semi_wallet::service::plan::service::Service as PlanService;
use semi_wallet::{config, router, AppState};
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let cfg = config::get_configuration().expect("cannot parse configuration");
    let repo = Repository::new();
    let db_pool = postgres::new_pg_pool(&cfg.db.dsn).await;
    let payment_service = PaymentService::new(
        db_pool.clone(),
        repo.clone(),
        &cfg.stripe.url,
        &cfg.stripe.secret,
    );
    let plan_service = PlanService::new(db_pool.clone(), repo.clone());
    let order_service =
        OrderService::new(db_pool.clone(), repo.clone(), plan_service, payment_service);
    let app_state = AppState { order_service };
    let shared_state = Arc::new(RwLock::new(app_state));
    let app = router::get_router(shared_state).await;
    let listener = tokio::net::TcpListener::bind(cfg.server.address)
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
