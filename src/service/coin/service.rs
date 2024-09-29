use sqlx::{Pool, Postgres};

use tracing::error;

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

    pub async fn coins_list(&self) -> Result<Vec<Coin>, CoinError> {
        let res = self.repo.get_all_coins(&self.db).await;
        if let Err(e) = res {
            error!("cannot acquire db conn due to err {e}");
            return Err(CoinError::Unexpected {
                message: "cannot get coins from db".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error>,
            });
        }
        let res = res.unwrap();
        let mut coins = Vec::with_capacity(res.len());
        for r in res {
            coins.push(Coin {
                id: r.id,
                symbol: r.symbol,
                name: r.name,
                logo: r.logo,
                network: r.network,
                decimals: r.decimals,
                description: r.description,
            });
        }
        Ok(coins)
    }
}
