use super::{db::Repository, models::Coin};

use sqlx::{Pool, Postgres};

impl Repository {
    pub async fn get_all_coins(&self, db: &Pool<Postgres>) -> Result<Vec<Coin>, sqlx::Error> {
        let coins = sqlx::query_as::<_, Coin>("SELECT * FROM coins")
            .fetch_all(db)
            .await?;
        Ok(coins)
    }
}
