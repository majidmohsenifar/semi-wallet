use redis::{self, AsyncCommands, FromRedisValue, RedisError};
use serde::{Deserialize, Serialize};

pub const COIN_REDIS_KEY_PREFIX: &str = "coin:";

#[derive(Serialize, Deserialize)]
pub struct PriceData {
    pub price: f64,
}

impl FromRedisValue for PriceData {
    fn from_byte_vec(vec: &[u8]) -> Option<Vec<Self>> {
        let val = serde_json::from_slice(vec).ok()?;
        Some(vec![val])
    }

    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        match v {
            redis::Value::SimpleString(s) => {
                let res = serde_json::from_str(s.as_str())?;
                Ok(res)
            }
            redis::Value::BulkString(bytes) => {
                let res = serde_json::from_slice(bytes)?;
                Ok(res)
            }
            _ => Err(redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Invalid type",
            ))),
        }
    }
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
        let key = format!("{}{}", COIN_REDIS_KEY_PREFIX, symbol);
        let val = PriceData { price };
        //TODO: handle this unwrap later
        let p = serde_json::to_string(&val).unwrap();
        conn.set(key, p).await?;
        Ok(())
    }

    pub async fn get_prices_for_symbols(
        &self,
        symbols: Vec<&str>,
    ) -> Result<Vec<PriceData>, RedisError> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .unwrap();
        let result: Vec<PriceData> = redis::pipe()
            .atomic()
            .mget(symbols)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }
}
