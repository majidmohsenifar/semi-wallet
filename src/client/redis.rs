use redis::{self, Client, RedisResult};

use crate::config;

pub async fn new_redis_client(redis_conf: config::RedisConfig) -> RedisResult<Client> {
    redis::Client::open(redis_conf.uri)
}
