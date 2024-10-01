use crate::{config::Settings, handler, AppState, SharedState};

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::{io, net::TcpListener, sync::RwLock};

use crate::client::postgres;
use crate::repository::db::Repository;
use crate::service::auth::service::Service as AuthService;
use crate::service::coin::service::Service as CoinService;
use crate::service::order::service::Service as OrderService;
use crate::service::payment::service::Service as PaymentService;
use crate::service::plan::service::Service as PlanService;

pub struct HttpServer {
    router: Router,
    listener: TcpListener,
}

impl HttpServer {
    pub async fn build(cfg: Settings) -> Self {
        //pub async fn run_server(cfg: Settings) {
        let repo = Repository::new();
        let db_pool = postgres::new_pg_pool(&cfg.db.dsn).await;
        let payment_service = PaymentService::new(
            db_pool.clone(),
            repo.clone(),
            &cfg.stripe.url,
            &cfg.stripe.secret,
        );
        let coin_service = CoinService::new(db_pool.clone(), repo.clone());
        let plan_service = PlanService::new(db_pool.clone(), repo.clone());
        let order_service =
            OrderService::new(db_pool.clone(), repo.clone(), plan_service, payment_service);
        let auth_service = AuthService::new(db_pool.clone(), repo.clone(), cfg.jwt.secret);

        let app_state = AppState {
            order_service,
            coin_service,
            auth_service,
        };
        let shared_state = Arc::new(RwLock::new(app_state));

        let router = get_router(shared_state).await;
        let listener = tokio::net::TcpListener::bind(cfg.server.address)
            .await
            .unwrap();
        //axum::serve(listener, router).await.unwrap();
        HttpServer { router, listener }
    }
    pub async fn run(self) -> Result<(), io::Error> {
        axum::serve(self.listener, self.router).await
    }
}

pub async fn get_router(shared_state: SharedState) -> Router {
    let order_routes = Router::new()
        .route("/create", post(handler::order::create_order))
        .route("/detail", get(handler::order::order_detail));

    let coin_routes = Router::new().route("/", get(handler::coin::coins_list));
    let auth_routes = Router::new()
        .route("/register", post(handler::auth::register))
        .route("/login", post(handler::auth::login));

    let api_routes = Router::new()
        .nest("/orders", order_routes)
        .nest("/coins", coin_routes)
        .nest("/auth", auth_routes);

    Router::new()
        .nest("/api/v1", api_routes)
        .with_state(shared_state)
}
