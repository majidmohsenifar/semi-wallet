use std::collections::HashMap;

use redis::{self, AsyncCommands, RedisError};
use serde::{Deserialize, Serialize};

pub const COIN_PRICE_REDIS_KEY_PREFIX: &str = "coin-price:";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub price: f64,
}

#[derive(Clone)]
pub struct PriceStorage {
    redis_client: redis::Client,
}

impl PriceStorage {
    pub fn new(redis_client: redis::Client) -> Self {
        Self { redis_client }
    }

    pub async fn set_price(&self, symbol: &str, price: f64) -> Result<(), RedisError> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .unwrap();
        let key = format!("{}{}", COIN_PRICE_REDIS_KEY_PREFIX, symbol);
        let val = PriceData { price };
        //TODO: handle this unwrap later
        let p = serde_json::to_string(&val).unwrap();
        conn.set(key, p).await?;
        Ok(())
    }

    pub async fn get_prices_for_symbols<'a>(
        &self,
        symbols: Vec<&'a str>,
    ) -> Result<HashMap<&'a str, PriceData>, RedisError> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .unwrap();
        let keys: Vec<String> = symbols
            .iter()
            .map(|&s| format!("{}{}", COIN_PRICE_REDIS_KEY_PREFIX, s))
            .collect();

        let result: redis::Value = redis::pipe()
            .atomic()
            .mget(&keys)
            .query_async(&mut conn)
            .await?;

        let prices_data = if let redis::Value::Array(values) = result {
            //the values is array containing 1 array, we are kinda flatten it
            if !values.is_empty() {
                let values = values[0].clone();
                match values {
                    redis::Value::Array(vals) => vals
                        .into_iter()
                        .map(|v| match v {
                            redis::Value::BulkString(d) => {
                                serde_json::from_slice::<PriceData>(&d).unwrap()
                            }
                            _ => PriceData { price: 0.0 },
                        })
                        .collect(),
                    _ => vec![PriceData { price: 0.0 }; keys.len()],
                }
            } else {
                vec![PriceData { price: 0.0 }; keys.len()]
            }
        } else {
            vec![PriceData { price: 0.0 }; keys.len()]
        };
        let prices: HashMap<&str, PriceData> = symbols.into_iter().zip(prices_data).collect();
        Ok(prices)
    }
}
