use super::{db::Repository, models::Coin};

use sqlx::{Pool, Postgres};

pub struct CreateCoinArgs {
    pub symbol: String,
    pub name: String,
    pub network: String,
    pub price_pair_symbol: Option<String>,
    pub logo: String,
    pub decimals: i16,
    pub description: String,
    pub contract_address: Option<String>,
}

impl Repository {
    pub async fn get_all_coins(&self, db: &Pool<Postgres>) -> Result<Vec<Coin>, sqlx::Error> {
        let coins = sqlx::query_as::<_, Coin>("SELECT * FROM coins ORDER BY id ASC")
            .fetch_all(db)
            .await?;
        Ok(coins)
    }

    pub async fn create_coin(
        &self,
        db: &Pool<Postgres>,
        args: CreateCoinArgs,
    ) -> Result<Coin, sqlx::Error> {
        let res = sqlx::query_as::<_, Coin>(
            "INSERT INTO coins (
            symbol,
            name,
            logo,
            network,
            price_pair_symbol,
            decimals,
            contract_address,
            description
            ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8
            ) RETURNING *",
        )
        .bind(args.symbol)
        .bind(args.name)
        .bind(args.logo)
        .bind(args.network)
        .bind(args.price_pair_symbol)
        .bind(args.decimals)
        .bind(args.contract_address)
        .bind(args.description)
        .fetch_one(db)
        .await?;
        Ok(res)
    }

    pub async fn get_coin_by_symbol_network(
        &self,
        db: &Pool<Postgres>,
        symbol: &str,
        network: &str,
    ) -> Result<Coin, sqlx::Error> {
        let res =
            sqlx::query_as::<_, Coin>("SELECT * from coins WHERE symbol = $1 AND network = $2")
                .bind(symbol)
                .bind(network)
                .fetch_one(db)
                .await?;
        Ok(res)
    }

    pub async fn get_not_null_price_pair_symbol_coins(
        &self,
        db: &Pool<Postgres>,
    ) -> Result<Vec<Coin>, sqlx::Error> {
        let coins = sqlx::query_as::<_, Coin>(
            "SELECT * FROM coins WHERE price_pair_symbol IS NOT NULL ORDER BY id ASC",
        )
        .fetch_all(db)
        .await?;
        Ok(coins)
    }
}
