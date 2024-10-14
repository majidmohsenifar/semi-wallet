use crate::{config::Settings, handler, middleware, AppState, SharedState};

use axum::{
    middleware as axum_middleware,
    routing::{delete, get, patch, post},
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
        handler::order::order_detail,
        handler::order::create_order,
        handler::auth::register,
        handler::auth::login,
        handler::coin::coins_list,
        handler::plan::plans_list,
        handler::user_coin::user_coins_list,
        handler::user_coin::create_user_coin,
        handler::user_coin::delete_user_coin,
        handler::user_coin::update_user_coin_address,
    ),
    components(schemas(
        //aliases
        crate::handler::response::ApiResponseCreateUserCoin, 
        crate::handler::response::ApiResponseUserCoinList,
        crate::handler::response::ApiResponseLogin,
        crate::handler::response::ApiResponseRegister,
        crate::handler::response::ApiResponseCoinList,
        crate::handler::response::ApiResponsePlanList,
        crate::handler::response::ApiResponseCreateOrder,
        crate::handler::response::ApiResponseOrderDetail,
        crate::handler::response::ApiResponseEmpty,

        crate::service::order::service::OrderDetailResult,
        crate::service::order::service::CreateOrderParams,
        crate::service::order::service::CreateOrderResult,
        crate::service::auth::service::RegisterParams,
        crate::service::auth::service::RegisterResult,
        crate::service::auth::service::LoginParams,
        crate::service::auth::service::LoginResult,
        crate::service::coin::service::Coin,
        crate::service::plan::service::Plan,
        crate::service::user_coin::service::CreateUserCoinParams,
        crate::service::user_coin::service::UserCoin,
        crate::handler::response::Empty,
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
        let repo = Repository::new();
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
            payment_service,
            user_plan_service,
            cfg.stripe.secret,
        );
        let auth_service = AuthService::new(db_pool.clone(), user_service, cfg.jwt.secret);
        let user_coin_service =
            UserCoinService::new(db_pool.clone(), repo.clone(), coin_service.clone());

        let app_state = AppState {
            order_service,
            coin_service,
            plan_service,
            auth_service,
            user_coin_service,
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
        .route("/register", post(handler::auth::register))
        .route("/login", post(handler::auth::login));

    let order_routes = Router::new()
        .route("/create", post(handler::order::create_order))
        .route("/detail", get(handler::order::order_detail))
        .layer(axum_middleware::from_fn_with_state(
            shared_state.clone(),
            middleware::jwt_auth::auth_middleware,
        ));

    let payments_routes = Router::new().route(
        "/callback/stripe",
        post(handler::payment::handle_stripe_webhook),
    );

    let coin_routes = Router::new().route("/", get(handler::coin::coins_list));

    let plan_routes = Router::new().route("/", get(handler::plan::plans_list));
    let user_coin_routes = Router::new()
        .route("/", get(handler::user_coin::user_coins_list))
        .route("/create", post(handler::user_coin::create_user_coin))
        .route("/delete", delete(handler::user_coin::delete_user_coin))
        .route(
            "/update-address",
            patch(handler::user_coin::update_user_coin_address),
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
        //.merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        // There is no need to create `RapiDoc::with_openapi` because the OpenApi is served
        // via SwaggerUi instead we only make rapidoc to point to the existing doc.
        //.merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        // Alternative to above
        // .merge(RapiDoc::with_openapi("/api-docs/openapi2.json", ApiDoc::openapi()).path("/rapidoc"))
        //.merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
        .nest("/api/v1", api_routes)
        .with_state(shared_state)
}
