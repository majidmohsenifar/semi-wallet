use sqlx::{Pool, Postgres};

use crate::repository::db::Repository;

use super::error::CoinError;

pub struct Service {
    db: Pool<Postgres>,
    repo: Repository,
}

#[derive(serde::Serialize)]
pub struct Coin {
    pub id: i64,
    pub symbol: String,
    pub name: String,
    pub logo: String,
    pub network: String,
    pub decimals: i16,
    pub description: String,
}

impl Service {
    pub fn new(db: Pool<Postgres>, repo: Repository) -> Self {
        Service { db, repo }
    }

    pub async fn coin_list(&self) -> Result<Vec<Coin>, CoinError> {
        todo!("handle this later");
    }
}
