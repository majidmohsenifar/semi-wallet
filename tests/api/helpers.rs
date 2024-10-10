use std::collections::BTreeMap;

use semi_wallet::{
    http_server::HttpServer,
    repository::{
        coin::CreateCoinArgs, db::Repository, models::Coin, models::User, user::CreateUserArgs,
    },
    service::auth::jwt,
};
use sqlx::{Connection, Executor, PgConnection, Pool, Postgres};
use uuid::Uuid;

use semi_wallet::service::auth::bcrypt;
use semi_wallet::{client::postgres, config};

use once_cell::sync::Lazy;
use wiremock::MockServer;

static TRACING: Lazy<()> = Lazy::new(|| {
    //let default_filter_level = "info".to_string();
    //let subscriber_name = "test".to_string();
    //// We cannot assign the output of `get_subscriber` to a variable based on the value of `TEST_LOG`
    //// because the sink is part of the type returned by `get_subscriber`, therefore they are not the
    //// same type. We could work around it, but this is the most straight-forward way of moving forward.
    //if std::env::var("TEST_LOG").is_ok() {
    //let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
    //init_subscriber(subscriber);
    //} else {
    //let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
    //init_subscriber(subscriber);
    //};
});

//TODO: try to implement this later
//static PLANS: Lazy<HashMap<&'static str, Plan>> = Lazy::new(|| {
//todo!();
//});

static COINS: Lazy<BTreeMap<&'static str, Coin>> = Lazy::new(|| {
    BTreeMap::from([
        (
            "BTC",
            Coin {
                id: 1,
                symbol: "BTC".to_string(),
                name: "Bitcoin".to_string(),
                network: "BTC".to_string(),
                logo: "btc.png".to_string(),
                decimals: 8,
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
                logo: "eth.png".to_string(),
                decimals: 18,
                description: Some("Ethereum is the second best".to_string()),
            },
        ),
        (
            "USDT_ETH",
            Coin {
                id: 3,
                symbol: "USDT".to_string(),
                name: "Tether".to_string(),
                network: "ETH".to_string(),
                logo: "usdt.png".to_string(),
                decimals: 18,
                description: Some("Tether is the third best".to_string()),
            },
        ),
        (
            "USDT_TRX",
            Coin {
                id: 4,
                symbol: "USDT".to_string(),
                name: "Tether".to_string(),
                network: "TRX".to_string(),
                logo: "usdt_trx.png".to_string(),
                decimals: 18,
                description: Some("Tether is the third best".to_string()),
            },
        ),
    ])
});

pub struct TestApp {
    pub address: String,
    pub db: Pool<Postgres>,
    pub repo: Repository,
    pub stripe_server: MockServer,
    pub cfg: config::Settings,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    Lazy::force(&COINS);

    let stripe_server = MockServer::start().await;
    let cfg = {
        let mut cfg = config::get_configuration().expect("failed to get configuration");
        let db_dsn = configure_db(&cfg.db).await;
        cfg.db.dsn = db_dsn;
        //consider the port 0, so the os will provide a free port
        cfg.server.address = "127.0.0.1:0".to_string();
        cfg.stripe.url = stripe_server.uri();
        cfg
    };
    let http_server = HttpServer::build(cfg.clone()).await;
    let address = format!("http://127.0.0.1:{}", http_server.port());
    tokio::spawn(http_server.run());
    let db = postgres::new_pg_pool(&cfg.db.dsn).await;
    let repo = Repository::new();

    TestApp {
        address,
        db,
        repo,
        stripe_server,
        cfg,
    }
}

impl TestApp {
    pub async fn get_jwt_token(&self, email: &str) -> (String, User) {
        //TODO: isn't it better to call register and login endpoint?
        let mut conn = self.db.acquire().await.unwrap();
        let encrypted_password = bcrypt::encrypt_password("12345678").unwrap();
        let user = self
            .repo
            .create_user(
                &mut conn,
                CreateUserArgs {
                    email: String::from(email),
                    password: encrypted_password,
                },
            )
            .await
            .unwrap();

        let token = jwt::create_jwt(self.cfg.jwt.secret.as_bytes(), String::from(email)).unwrap();
        (token, user)
    }

    pub async fn insert_coins(&self) {
        for (_, c) in COINS.iter() {
            self.repo
                .create_coin(
                    &self.db,
                    CreateCoinArgs {
                        symbol: c.symbol.clone(),
                        name: c.name.clone(),
                        network: c.network.clone(),
                        logo: c.logo.clone(),
                        decimals: c.decimals,
                        description: c.description.clone().unwrap(),
                    },
                )
                .await
                .unwrap();
        }
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
    let db_pool = postgres::new_pg_pool(&db_dsn).await;
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("failed to run migrations");

    db_dsn
}
