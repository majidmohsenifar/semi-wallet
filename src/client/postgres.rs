use std::time::Duration;

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub async fn new_pg_pool(db_dsn: &str) -> Result<Pool<Postgres>, sqlx::Error> {
    // set up connection pool
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(db_dsn)
        .await
}
