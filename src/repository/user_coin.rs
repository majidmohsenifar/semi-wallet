use sqlx::types::BigDecimal;
use sqlx::{Pool, Postgres};

use super::{db::Repository, models::UserCoin};
use sqlx::Row;

pub struct CreateUserCoinArgs<'a> {
    pub user_id: i64,
    pub coin_id: i64,
    pub symbol: &'a str,
    pub network: &'a str,
    pub address: &'a str,
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
        args: CreateUserCoinArgs<'_>,
    ) -> Result<i64, sqlx::Error> {
        let res = sqlx::query(
            "INSERT INTO users_coins
                (
                    user_id,
                    coin_id,
                    address,
                    symbol,
                    network
                ) VALUES(
                $1, $2, $3, $4, $5
                ) ON CONFLICT (user_id, coin_id, address, network) DO UPDATE SET coin_id = EXCLUDED.coin_id RETURNING id",
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

    pub async fn update_user_coin_amount(
        &self,
        db: &Pool<Postgres>,
        id: i64,
        amount: BigDecimal,
    ) -> Result<u64, sqlx::Error> {
        let res = sqlx::query(
            "
            UPDATE users_coins 
            SET amount = $2, 
            amount_updated_at = NOW()
            WHERE id = $1",
        )
        .bind(id)
        .bind(amount)
        .execute(db)
        .await?;
        Ok(res.rows_affected())
    }

    pub async fn get_user_coins_by_user_ids(
        &self,
        db: &Pool<Postgres>,
        user_ids: Vec<i64>,
    ) -> Result<Vec<UserCoin>, sqlx::Error> {
        let user_coins = sqlx::query_as::<_, UserCoin>(
            "SELECT * FROM users_coins WHERE user_id IN (SELECT unnest($1::bigint[]))",
        )
        .bind(user_ids)
        .fetch_all(db)
        .await?;
        Ok(user_coins)
    }

    pub async fn get_user_coins_by_user_ids_coin_id(
        &self,
        db: &Pool<Postgres>,
        user_ids: Vec<i64>,
        coin_id: i64,
    ) -> Result<Vec<UserCoin>, sqlx::Error> {
        let user_coins = sqlx::query_as::<_, UserCoin>(
            "SELECT * FROM users_coins WHERE user_id IN (SELECT unnest($1::bigint[])) AND coin_id = $2",
        )
        .bind(user_ids)
        .bind(coin_id)
        .fetch_all(db)
        .await?;
        Ok(user_coins)
    }
}
