use redis::{self, Client, RedisResult};

pub async fn new_redis_client(redis_uri: &str) -> RedisResult<Client> {
    redis::Client::open(redis_uri)
}
