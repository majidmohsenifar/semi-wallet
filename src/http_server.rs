use crate::{config::Settings, handler::{self,api::middleware}, AppState, SharedState};

use axum::{
    middleware as axum_middleware,
    routing::{delete, get, patch, post},
    Router, 
    http::Method,
};
use std::sync::Arc;
use tokio::{io, net::TcpListener, sync::RwLock};
use tower_http::cors::{CorsLayer,Any};

use crate::client::postgres;
use crate::repository::db::Repository;
use crate::service::auth::service::Service as AuthService;
use crate::service::coin::service::Service as CoinService;
use crate::service::order::service::Service as OrderService;
use crate::service::payment::service::Service as PaymentService;
use crate::service::plan::service::Service as PlanService;
use crate::service::user::service::Service as UserService;
use crate::service::user_coin::service::Service as UserCoinService;
use crate::service::user_plan::service::Service as UserPlanService;
use utoipa::{
    openapi::{
        self,
        security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    },
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

pub struct HttpServer {
    router: Router,
    listener: TcpListener,
    port: u16,
}

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    paths(
        handler::api::order::create_order,
        handler::api::order::order_detail,
        handler::api::order::user_orders_list,
        handler::api::auth::register,
        handler::api::auth::login,
        handler::api::coin::coins_list,
        handler::api::plan::plans_list,
        handler::api::user_coin::user_coins_list,
        handler::api::user_coin::create_user_coin,
        handler::api::user_coin::delete_user_coin,
        handler::api::user_coin::update_user_coin_address,
        handler::api::payment::payment_providers,
    ),
    components(schemas(
        //aliases
        crate::handler::api::response::ApiResponseCreateUserCoin, 
        crate::handler::api::response::ApiResponseUserCoinsList,
        crate::handler::api::response::ApiResponseLogin,
        crate::handler::api::response::ApiResponseRegister,
        crate::handler::api::response::ApiResponseCoinsList,
        crate::handler::api::response::ApiResponsePlansList,
        crate::handler::api::response::ApiResponseCreateOrder,
        crate::handler::api::response::ApiResponseOrderDetail,
        crate::handler::api::response::ApiResponseUserOrdersList,
        crate::handler::api::response::ApiResponsePaymentProvidersList,
        crate::handler::api::response::ApiResponseEmpty,

        crate::service::order::service::OrderDetailResult,
        crate::service::order::service::CreateOrderParams,
        crate::service::order::service::GetUserOrdersListParams,
        crate::service::order::service::CreateOrderResult,
        crate::service::order::service::Order,
        crate::service::auth::service::RegisterParams,
        crate::service::auth::service::RegisterResult,
        crate::service::auth::service::LoginParams,
        crate::service::auth::service::LoginResult,
        crate::service::coin::service::Coin,
        crate::service::plan::service::Plan,
        crate::service::payment::service::PaymentProvider,
        crate::service::user_coin::service::CreateUserCoinParams,
        crate::service::user_coin::service::UserCoin,
        crate::handler::api::response::Empty,
    )),
tags(
(name = "semi-wallet", description = "semi wallet API")
)
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_jwt_token",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}

impl HttpServer {
    pub async fn build(cfg: Settings) -> Self {
        //pub async fn run_server(cfg: Settings) {
        let repo = Repository::default();
        let db_pool = postgres::new_pg_pool(&cfg.db.dsn).await;
        let payment_service = PaymentService::new(
            db_pool.clone(),
            repo.clone(),
            &cfg.stripe.url,
            &cfg.stripe.secret,
        );
        let user_service = UserService::new(db_pool.clone(), repo.clone());
        let coin_service = CoinService::new(db_pool.clone(), repo.clone());
        let plan_service = PlanService::new(db_pool.clone(), repo.clone());
        let user_plan_service = UserPlanService::new(db_pool.clone(), repo.clone());
        let order_service = OrderService::new(
            db_pool.clone(),
            repo.clone(),
            plan_service.clone(),
            payment_service.clone(),
            user_plan_service.clone(),
            cfg.stripe.secret,
        );
        let auth_service = AuthService::new(db_pool.clone(), user_service, cfg.jwt.secret);
        let user_coin_service =
            UserCoinService::new(db_pool.clone(), repo.clone(), coin_service.clone(),user_plan_service.clone());

        let app_state = AppState {
            order_service,
            coin_service,
            plan_service,
            auth_service,
            user_coin_service,
            payment_service,
        };
        let shared_state = Arc::new(RwLock::new(app_state));

        let router = get_router(shared_state).await;
        let listener = tokio::net::TcpListener::bind(cfg.server.address)
            .await
            .unwrap();
        let port = listener.local_addr().unwrap().port();
        //axum::serve(listener, router).await.unwrap();
        HttpServer {
            router,
            listener,
            port,
        }
    }
    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run(self) -> Result<(), io::Error> {
        axum::serve(self.listener, self.router).await
    }
}

pub async fn get_router(shared_state: SharedState) -> Router {
    let auth_routes = Router::new()
        .route("/register", post(handler::api::auth::register))
        .route("/login", post(handler::api::auth::login));

    let order_routes = Router::new()
        .route("/", get(handler::api::order::user_orders_list))
        .route("/create", post(handler::api::order::create_order))
        .route("/detail", get(handler::api::order::order_detail))
        .layer(axum_middleware::from_fn_with_state(
            shared_state.clone(),
            middleware::jwt_auth::auth_middleware,
        ));

    let payments_routes = Router::new()
        .route("/callback/stripe",post(handler::api::payment::handle_stripe_webhook))
        .route("/providers", get(handler::api::payment::payment_providers));

    let coin_routes = Router::new().route("/", get(handler::api::coin::coins_list));

    let plan_routes = Router::new().route("/", get(handler::api::plan::plans_list));
    let user_coin_routes = Router::new()
        .route("/", get(handler::api::user_coin::user_coins_list))
        .route("/create", post(handler::api::user_coin::create_user_coin))
        .route("/delete", delete(handler::api::user_coin::delete_user_coin))
        .route(
            "/update-address",
            patch(handler::api::user_coin::update_user_coin_address),
        )
        .layer(axum_middleware::from_fn_with_state(
            shared_state.clone(),
            middleware::jwt_auth::auth_middleware,
        ));

    let api_routes = Router::new()
        .nest("/auth", auth_routes)
        .nest("/orders", order_routes)
        .nest("/coins", coin_routes)
        .nest("/plans", plan_routes)
        .nest("/payments", payments_routes)
        .nest("/user-coins", user_coin_routes);

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api/v1", api_routes)
        //.layer(TraceLayer::new_for_http()
            ////.make_span_with(new_make_span)
            //.on_failure(|error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
            //tracing::error!("error: {}", error)
        //}))
        .layer(CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET,Method::POST,Method::PATCH,Method::PUT])
                .allow_headers(Any),//TODO: should we let Any header to be passed?
        )
        .with_state(shared_state)
}
