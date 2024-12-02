use std::process;

use semi_wallet::client::{postgres, redis};
use semi_wallet::config;
use semi_wallet::repository::db::Repository;
use semi_wallet::service::coin::price_manager::{PriceManager, PRICE_PROVIDER_BINANCE};
use semi_wallet::service::coin::price_storage::PriceStorage;
use semi_wallet::service::coin::service::Service as CoinService;
use semi_wallet::telemetry::{get_subscriber, init_subscriber};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SubscribeRequest<'a> {
    id: u32,
    method: &'a str,
    params: Vec<&'a str>,
}

#[tokio::main]
async fn main() {
    let cfg = config::Settings::new();
    let cfg = match cfg {
        Ok(cfg) => cfg,
        Err(e) => {
            tracing::error!("cannot create configs to err: {}", e);
            process::exit(1);
        }
    };
    let subscriber = get_subscriber("semi-wallet-cli".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let repo = Repository::default();
    let db_pool = postgres::new_pg_pool(&cfg.db.dsn)
        .await
        .expect("cannot create db_pool");

    let coin_service = CoinService::new(db_pool.clone(), repo.clone());
    let coins = coin_service
        .get_not_null_price_pair_symbol_coins()
        .await
        .expect("cannot get coins");

    if coins.is_empty() {
        panic!("coins is empty");
    }

    let redis_client = redis::new_redis_client(&cfg.redis.uri)
        .await
        .expect("cannot create redis client");
    let price_storage = PriceStorage::new(redis_client);

    let price_manager = PriceManager::new(
        price_storage,
        PRICE_PROVIDER_BINANCE,
        cfg.binance.clone(),
        coins,
    )
    .expect("cannot create price manager");

    price_manager.run_update_prices().await;
}
