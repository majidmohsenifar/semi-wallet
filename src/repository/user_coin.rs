use sqlx::{Pool, Postgres};

use super::{db::Repository, models::UserCoin};
use sqlx::Row;

pub struct CreateUserCoinArgs {
    pub user_id: i64,
    pub coin_id: i64,
    pub symbol: String,
    pub network: String,
    pub address: String,
}

impl Repository {
    pub async fn get_user_coins_by_user_id(
        &self,
        db: &Pool<Postgres>,
        user_id: i64,
    ) -> Result<Vec<UserCoin>, sqlx::Error> {
        let user_coins = sqlx::query_as::<_, UserCoin>(
            "SELECT * FROM users_coins WHERE user_id = $1 ORDER BY id ASC",
        )
        .bind(user_id)
        .fetch_all(db)
        .await?;
        Ok(user_coins)
    }

    pub async fn create_user_coin(
        &self,
        db: &Pool<Postgres>,
        args: CreateUserCoinArgs,
    ) -> Result<i64, sqlx::Error> {
        let res = sqlx::query(
            "INSERT INTO users_coins
                (
                    user_id,
                    coin_id,
                    address,
                    symbol,
                    network,
                    created_at,
                    updated_at
                ) VALUES(
                $1, $2, $3, $4, $5, NOW(),NOW()
                ) RETURNING id",
        )
        .bind(args.user_id)
        .bind(args.coin_id)
        .bind(args.address)
        .bind(args.symbol)
        .bind(args.network)
        .fetch_one(db)
        .await?;
        Ok(res.get::<i64, _>(0))
    }

    pub async fn delete_user_coin(
        &self,
        db: &Pool<Postgres>,
        user_id: i64,
        id: i64,
    ) -> Result<u64, sqlx::Error> {
        let res = sqlx::query("DELETE FROM users_coins WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(db)
            .await?;
        Ok(res.rows_affected())
    }

    pub async fn update_user_coin_address(
        &self,
        db: &Pool<Postgres>,
        user_id: i64,
        id: i64,
        address: &str,
    ) -> Result<u64, sqlx::Error> {
        let res = sqlx::query(
            "
            UPDATE users_coins 
            SET address = $3 
            WHERE id = $1 AND user_id = $2",
        )
        .bind(id)
        .bind(user_id)
        .bind(address)
        .execute(db)
        .await?;
        Ok(res.rows_affected())
    }
}
