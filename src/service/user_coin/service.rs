use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use utoipa::ToSchema;
use validator::Validate;

use crate::repository::{db::Repository, models::User, user_coin::CreateUserCoinArgs};

use crate::service::coin::service::Service as CoinService;

use super::error::UserCoinError;

pub struct Service {
    db: Pool<Postgres>,
    repo: Repository,
    coin_service: CoinService,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserCoin {
    pub id: i64,
    pub coin_id: i64,
    pub address: String,
    pub symbol: String,
    pub network: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateUserCoinParams {
    #[validate(length(min = 1))]
    pub address: String,
    #[validate(length(min = 1))]
    pub symbol: String,
    pub network: Option<String>,
}

impl Service {
    pub fn new(db: Pool<Postgres>, repo: Repository, coin_service: CoinService) -> Self {
        Service {
            db,
            repo,
            coin_service,
        }
    }

    pub async fn get_user_coins_list(&self, user: User) -> Result<Vec<UserCoin>, UserCoinError> {
        let res = self
            .repo
            .get_user_coins_by_user_id(&self.db, user.id)
            .await
            .map_err(|e| UserCoinError::Unexpected {
                message: "cannot get user coins from db".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            })?;
        //let mut user_coins = Vec::with_capacity(res.len());
        let user_coins = res
            .into_iter()
            .map(|uc| UserCoin {
                id: uc.id,
                coin_id: uc.coin_id,
                address: uc.address,
                symbol: uc.symbol,
                network: uc.network,
                created_at: uc.created_at.timestamp(),
                updated_at: uc.updated_at.timestamp(),
            })
            .collect();

        Ok(user_coins)
    }

    pub async fn create_user_coin(
        &self,
        user: User,
        mut params: CreateUserCoinParams,
    ) -> Result<UserCoin, UserCoinError> {
        //TODO: we should validate the address
        if params.network.is_none() || params.network.clone().unwrap().trim().is_empty() {
            params.network = Some(params.symbol.clone());
        }
        let coin = self
            .coin_service
            .get_coin_by_symbol_network(
                &params.symbol.to_uppercase(),
                &params.network.unwrap().to_uppercase(),
            )
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => UserCoinError::CoinOrNetworkNotFound,
                other => UserCoinError::Unexpected {
                    message: "cannot get coin by symbol or network".to_string(),
                    source: Box::new(other) as Box<dyn std::error::Error + Send + Sync>,
                },
            })?;

        let id = self
            .repo
            .create_user_coin(
                &self.db,
                CreateUserCoinArgs {
                    user_id: user.id,
                    coin_id: coin.id,
                    symbol: coin.symbol.clone(),
                    network: coin.network.clone(),
                    address: params.address.clone(),
                },
            )
            .await
            .map_err(|e| UserCoinError::Unexpected {
                message: "cannot create user coin".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            })?;

        Ok(UserCoin {
            id,
            coin_id: coin.id,
            address: params.address,
            symbol: coin.symbol,
            network: coin.network,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        })
    }

    pub async fn delete_user_coin(
        &self,
        user: User,
        user_coin_id: i64,
    ) -> Result<(), UserCoinError> {
        let rows_affected = self
            .repo
            .delete_user_coin(&self.db, user.id, user_coin_id)
            .await
            .map_err(|e| UserCoinError::Unexpected {
                message: "cannot delete user coin".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            })?;
        if rows_affected == 0 {
            return Err(UserCoinError::UserCoinNotFound);
        }
        Ok(())
    }

    pub async fn update_user_coin_address(
        &self,
        user: User,
        user_coin_id: i64,
        address: &str,
    ) -> Result<(), UserCoinError> {
        //TODO: validate address here
        let rows_affected = self
            .repo
            .update_user_coin_address(&self.db, user.id, user_coin_id, address)
            .await
            .map_err(|e| UserCoinError::Unexpected {
                message: "cannot delete user coin".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            })?;
        if rows_affected == 0 {
            return Err(UserCoinError::UserCoinNotFound);
        }
        Ok(())
    }
}
