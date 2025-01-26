use std::{collections::{BTreeMap, HashMap}, sync::LazyLock};

use redis::{self, Client};
use semi_wallet::{
    http_server::HttpServer,
    repository::{
        db::Repository, models::Coin, models::User, user::CreateUserArgs,
        user_coin::CreateUserCoinArgs,
    },
    service::auth::jwt,
    telemetry::{get_subscriber, init_subscriber},
};
use sqlx::{Connection, Executor, PgConnection, Pool, Postgres};
use uuid::Uuid;

use semi_wallet::service::auth::bcrypt;
use semi_wallet::{client::postgres, config};

use wiremock::MockServer;
use ws_mock::ws_mock_server::WsMockServer;

static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    // We cannot assign the output of `get_subscriber` to a variable based on the value of `TEST_LOG`
    // because the sink is part of the type returned by `get_subscriber`, therefore they are not the
    // same type. We could work around it, but this is the most straight-forward way of moving forward.
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub static COINS: LazyLock<BTreeMap<&'static str, Coin>> = LazyLock::new(|| {
    BTreeMap::from([
        (
            "BTC",
            Coin {
                id: 1,
                symbol: "BTC".to_string(),
                name: "Bitcoin".to_string(),
                network: "BTC".to_string(),
                price_pair_symbol: Some("BTC-USDT".to_string()),
                logo: "btc.png".to_string(),
                decimals: 8,
                contract_address: None,
                description: Some("Bitcoin is the best".to_string()),
            },
        ),
        (
            "ETH",
            Coin {
                id: 2,
                symbol: "ETH".to_string(),
                name: "Ethereum".to_string(),
                network: "ETH".to_string(),
                price_pair_symbol: Some("ETH-USDT".to_string()),
                logo: "eth.png".to_string(),
                decimals: 18,
                contract_address: None,
                description: Some("Ethereum is the second best".to_string()),
            },
        ),
        (
            "SOL",
            Coin {
                id: 3,
                symbol: "SOL".to_string(),
                name: "Solana".to_string(),
                network: "SOL".to_string(),
                logo: "sol.png".to_string(),
                price_pair_symbol: Some("SOL-USDT".to_string()),
                decimals: 9,
                contract_address: None,
                description: Some("Solana is the third best".to_string()),
            },
        ),
        (
            "TRX",
            Coin {
                id: 4,
                symbol: "TRX".to_string(),
                name: "Tron".to_string(),
                network: "TRX".to_string(),
                price_pair_symbol: Some("TRX-USDT".to_string()),
                logo: "trx.png".to_string(),
                decimals: 6,
                contract_address: None,
                description: Some("Trx is the fourth best".to_string()),
            },
        ),
        (
            "USDT_ETH",
            Coin {
                id: 5,
                symbol: "USDT".to_string(),
                name: "Tether".to_string(),
                network: "ETH".to_string(),
                price_pair_symbol: None,
                logo: "usdt.png".to_string(),
                decimals: 6,
                contract_address: Some("0xdac17f958d2ee523a2206206994597c13d831ec7".to_string()),
                description: Some("Tether is the best token".to_string()),
            },
        ),
        (
            "USDT_TRX",
            Coin {
                id: 6,
                symbol: "USDT".to_string(),
                name: "Tether".to_string(),
                network: "TRX".to_string(),
                price_pair_symbol: None,
                logo: "usdt_trx.png".to_string(),
                decimals: 6,
                contract_address: Some("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string()),
                description: Some("Tether is the best token".to_string()),
            },
        ),
    ])
});

pub struct TestApp<'a> {
    pub address: String,
    pub db: Pool<Postgres>,
    pub repo: Repository,
    pub cfg: config::Settings,
    pub stripe_server: MockServer,
    pub redis_client: Client,
    pub binance_ws_server: WsMockServer,
    pub nodes: HashMap<&'a str, MockServer>,
}

pub async fn spawn_app<'a>() -> TestApp<'a> {
    LazyLock::force(&TRACING);
    LazyLock::force(&COINS);

    let stripe_server = MockServer::start().await;
    let btc_node = MockServer::start().await;
    let eth_node = MockServer::start().await;
    let sol_node = MockServer::start().await;
    let trx_node = MockServer::start().await;
    let binance_ws_server = WsMockServer::start().await;
    let cfg = {
        let mut cfg = config::Settings::new().expect("cannot parse configuration");
        let db_dsn = configure_db(&cfg.db).await;
        cfg.db.dsn = db_dsn;
        //consider the port 0, so the os will provide a free port
        cfg.server.address = "127.0.0.1:0".to_string();
        cfg.stripe.url = stripe_server.uri();

        cfg.btc.url = btc_node.uri();
        cfg.eth.url = eth_node.uri();
        cfg.sol.url = sol_node.uri();
        cfg.trx.url = trx_node.uri();

        cfg.binance.ws_url = binance_ws_server.uri().await;

        cfg
    };
    let nodes = HashMap::from([
        ("BTC", btc_node),
        ("ETH", eth_node),
        ("SOL", sol_node),
        ("TRX", trx_node),
    ]);
    let http_server = HttpServer::build(cfg.clone()).await;
    let address = format!("http://127.0.0.1:{}", http_server.port());
    tokio::spawn(http_server.run());
    let db = postgres::new_pg_pool(&cfg.db.dsn)
        .await
        .expect("cannot create db_pool");
    let repo = Repository::default();
    let redis_client = semi_wallet::client::redis::new_redis_client(cfg.redis.clone())
        .await
        .unwrap();

    TestApp {
        address,
        db,
        repo,
        cfg,
        stripe_server,
        redis_client,
        binance_ws_server,
        nodes,
    }
}

impl<'a> TestApp<'a> {
    pub async fn get_jwt_token_and_user(&self, email: &str) -> (String, User) {
        let mut conn = self.db.acquire().await.unwrap();
        let encrypted_password = bcrypt::encrypt_password("12345678").unwrap();
        let user = self
            .repo
            .create_user(
                &mut conn,
                CreateUserArgs {
                    email,
                    password: &encrypted_password,
                },
            )
            .await
            .unwrap();

        let token = jwt::create_jwt(self.cfg.jwt.secret.as_bytes(), String::from(email)).unwrap();
        (token, user)
    }

    pub async fn create_user_coin(
        &self,
        user_id: i64,
        symbol: &str,
        network: &str,
        address: &str,
    ) -> i64 {
        let map_key = if symbol != network {
            &format!("{}_{}", symbol, network)
        } else {
            symbol
        };
        let coin = COINS.get(map_key).unwrap();
        self.repo
            .create_user_coin(
                &self.db,
                CreateUserCoinArgs {
                    user_id,
                    coin_id: coin.id,
                    symbol,
                    network,
                    address,
                },
            )
            .await
            .unwrap()
    }
}

async fn configure_db(db_cfg: &config::DbConfig) -> String {
    let db_name = Uuid::new_v4().to_string();
    let db_url = url::Url::parse(&db_cfg.dsn).expect("cannot parse db dsn");
    let db_dsn_without_database = format!(
        "postgres://{username}:{password}@{host}:{port}?sslmode=disable",
        username = db_url.username(),
        password = db_url.password().expect("empty password"),
        host = db_url.host().expect("empty host"),
        port = db_url.port().expect("empty port"),
    );

    let mut conn = PgConnection::connect(&db_dsn_without_database)
        .await
        .expect("cannot connect without db");

    conn.execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("failed to create database");

    let db_dsn = format!(
        "postgres://{username}:{password}@{host}:{port}/{db}?sslmode=disable",
        username = db_url.username(),
        password = db_url.password().expect("empty password"),
        host = db_url.host().expect("empty host"),
        port = db_url.port().expect("empty port"),
        db = db_name,
    );
    let db_pool = postgres::new_pg_pool(&db_dsn)
        .await
        .expect("cannot create db_pool");
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("failed to run migrations");

    db_dsn
}
