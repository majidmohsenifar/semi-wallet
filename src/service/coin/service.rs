use sqlx::{Pool, Postgres};

use tracing::error;
use utoipa::ToSchema;

use crate::repository::{db::Repository, models::Coin as CoinModel};

use super::error::CoinError;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Service {
    db: Pool<Postgres>,
    repo: Repository,
}

#[derive(Serialize, Deserialize, ToSchema)]
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
            tracing::error!("cannot get_all_coins due to err: {}", e);
            return Err(CoinError::Unexpected {
                message: "cannot get coins from db".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            });
        }
        let res = res.unwrap();
        let mut coins = Vec::with_capacity(res.len());
        for c in res {
            coins.push(Coin {
                id: c.id,
                symbol: c.symbol,
                name: c.name,
                logo: c.logo,
                network: c.network,
                decimals: c.decimals,
                description: c.description.unwrap_or("".to_string()),
            });
        }
        Ok(coins)
    }

    pub async fn get_coin_by_symbol_network(
        &self,
        symbol: &str,
        network: &str,
    ) -> Result<CoinModel, sqlx::Error> {
        self.repo
            .get_coin_by_symbol_network(&self.db, symbol, network)
            .await
    }
}
