use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use utoipa::ToSchema;
use validator::Validate;

use crate::repository::{
    db::Repository, models::User, models::UserCoin as UserCoinModel, user_coin::CreateUserCoinArgs,
};

use crate::service::coin::service::Service as CoinService;
use crate::service::user_plan::service::Service as UserPlanService;

use super::error::UserCoinError;

pub struct Service {
    db: Pool<Postgres>,
    repo: Repository,
    coin_service: CoinService,
    user_plan_service: UserPlanService,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserCoin {
    pub id: i64,
    pub coin_id: i64,
    pub address: String,
    pub symbol: String,
    pub network: String,
    pub amount: Option<f64>,
    pub amount_updated_at: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateUserCoinParams {
    #[validate(length(min = 32, message = "must be at least 32 characters"))]
    pub address: String,
    #[validate(length(min = 2, message = "must be at least 2 characters"))]
    pub symbol: String,
    pub network: Option<String>,
}

impl Service {
    pub fn new(
        db: Pool<Postgres>,
        repo: Repository,
        coin_service: CoinService,
        user_plan_service: UserPlanService,
    ) -> Self {
        Service {
            db,
            repo,
            coin_service,
            user_plan_service,
        }
    }

    pub async fn get_user_coins_list(&self, user: User) -> Result<Vec<UserCoin>, UserCoinError> {
        let res = self
            .repo
            .get_user_coins_by_user_id(&self.db, user.id)
            .await
            .map_err(|e| {
                tracing::error!("cannot get_user_coins_by_user_id due to err: {}", e);
                UserCoinError::Unexpected {
                    message: "cannot get user coins from db".to_string(),
                    source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
                }
            })?;
        let user_coins = res
            .into_iter()
            .map(|uc| {
                let mut amount = None;
                if let Some(bd) = uc.amount {
                    amount = bd.to_f64();
                }
                let mut amount_updated_at = None;
                if let Some(updated_at) = uc.amount_updated_at {
                    amount_updated_at = Some(updated_at.timestamp());
                }
                UserCoin {
                    id: uc.id,
                    coin_id: uc.coin_id,
                    address: uc.address,
                    symbol: uc.symbol,
                    network: uc.network,
                    amount,
                    amount_updated_at,
                    created_at: uc.created_at.timestamp(),
                }
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
                e => {
                    tracing::error!("cannot get_coin_by_symbol_network due to err: {}", e);
                    UserCoinError::Unexpected {
                        message: "cannot get coin by symbol or network".to_string(),
                        source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
                    }
                }
            })?;

        let user_plan = self
            .user_plan_service
            .get_user_plan_by_user_id(&self.db, user.id)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => UserCoinError::UserPlanNotFound,
                e => {
                    tracing::error!("cannot get_user_plan_by_user_id due to err: {}", e);
                    UserCoinError::Unexpected {
                        message: "cannot check if user has active plan".to_string(),
                        source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
                    }
                }
            })?;
        if user_plan.expires_at < chrono::Utc::now() {
            return Err(UserCoinError::UserPlanExpired);
        }

        let id = self
            .repo
            .create_user_coin(
                &self.db,
                CreateUserCoinArgs {
                    user_id: user.id,
                    coin_id: coin.id,
                    symbol: &coin.symbol,
                    network: &coin.network,
                    address: &params.address,
                },
            )
            .await
            .map_err(|e| {
                tracing::error!("cannot create_user_coin due to err: {}", e);
                UserCoinError::Unexpected {
                    message: "cannot create user coin".to_string(),
                    source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
                }
            })?;

        Ok(UserCoin {
            id,
            coin_id: coin.id,
            address: params.address,
            symbol: coin.symbol,
            network: coin.network,
            amount: None,
            amount_updated_at: None,
            created_at: chrono::Utc::now().timestamp(),
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
            .map_err(|e| {
                tracing::error!("cannot delete_user_coin due to err: {}", e);
                UserCoinError::Unexpected {
                    message: "cannot delete user coin".to_string(),
                    source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
                }
            })?;
        if rows_affected == 0 {
            return Err(UserCoinError::UserCoinNotFound);
        }
        Ok(())
    }

    pub async fn update_user_coin_address(
        &self,
        user: User,
        id: i64,
        address: &str,
    ) -> Result<(), UserCoinError> {
        //TODO: validate address here
        let rows_affected = self
            .repo
            .update_user_coin_address(&self.db, user.id, id, address)
            .await
            .map_err(|e| {
                tracing::error!("cannot update_user_coin_address due to err: {}", e);
                UserCoinError::Unexpected {
                    message: "cannot update user coin address".to_string(),
                    source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
                }
            })?;
        if rows_affected == 0 {
            return Err(UserCoinError::UserCoinNotFound);
        }
        Ok(())
    }

    pub async fn get_user_coins_by_user_ids(
        &self,
        user_ids: Vec<i64>,
    ) -> Result<Vec<UserCoinModel>, sqlx::Error> {
        self.repo
            .get_user_coins_by_user_ids(&self.db, user_ids)
            .await
    }

    pub async fn get_user_coins_by_user_ids_coin_id(
        &self,
        user_ids: Vec<i64>,
        coin_id: i64,
    ) -> Result<Vec<UserCoinModel>, sqlx::Error> {
        self.repo
            .get_user_coins_by_user_ids_coin_id(&self.db, user_ids, coin_id)
            .await
    }

    pub async fn update_user_coin_amount(&self, id: i64, amount: f64) -> Result<(), UserCoinError> {
        let amount = BigDecimal::from_f64(amount).ok_or(UserCoinError::InvalidAmount)?;
        let rows_affected = self
            .repo
            .update_user_coin_amount(&self.db, id, amount)
            .await
            .map_err(|e| {
                tracing::error!("cannot update_user_coin_amount due to err: {}", e);
                UserCoinError::Unexpected {
                    message: "cannot update user coin amount".to_string(),
                    source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
                }
            })?;
        if rows_affected == 0 {
            return Err(UserCoinError::UserCoinNotFound);
        }
        Ok(())
    }
}
