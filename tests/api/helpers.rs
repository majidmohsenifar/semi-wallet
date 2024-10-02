use semi_wallet::{http_server::HttpServer, repository::db::Repository};
use sqlx::{Connection, Executor, PgConnection, Pool, Postgres};
use uuid::Uuid;

use semi_wallet::{client::postgres, config};

pub struct TestApp {
    pub address: String,
    pub db: Pool<Postgres>,
    pub repo: Repository,
}

pub async fn spawn_app() -> TestApp {
    let cfg = {
        let mut cfg = config::get_configuration().expect("failed to get configuration");
        let db_dsn = configure_db(&cfg.db).await;
        cfg.db.dsn = db_dsn;
        //consider the port 0, so the os will provide a free port
        cfg.server.address = "127.0.0.1:0".to_string();
        cfg
    };
    //TODO: maybe we need tracing here too
    let http_server = HttpServer::build(cfg.clone()).await;
    let address = format!("http://127.0.0.1:{}", http_server.port());
    tokio::spawn(http_server.run());
    let db = postgres::new_pg_pool(&cfg.db.dsn).await;
    TestApp {
        address,
        db,
        repo: Repository::new(),
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
