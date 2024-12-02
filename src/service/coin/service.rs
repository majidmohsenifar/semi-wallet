use sqlx::{Pool, Postgres};

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
        let res = self
            .repo
            .get_all_coins(&self.db)
            .await
            .map_err(|e| CoinError::Unexpected {
                message: "cannot get all coins".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            })?;
        let coins = res
            .into_iter()
            .map(|c| Coin {
                id: c.id,
                symbol: c.symbol,
                name: c.name,
                logo: c.logo,
                network: c.network,
                decimals: c.decimals,
                description: c.description.unwrap_or("".to_string()),
            })
            .collect();
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

    pub async fn get_all_coins(&self) -> Result<Vec<CoinModel>, sqlx::Error> {
        self.repo.get_all_coins(&self.db).await
    }

    pub async fn get_not_null_price_pair_symbol_coins(
        &self,
    ) -> Result<Vec<CoinModel>, sqlx::Error> {
        self.repo.get_all_coins(&self.db).await
    }
}
