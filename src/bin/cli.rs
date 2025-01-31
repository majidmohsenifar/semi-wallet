use clap::{Parser, Subcommand};
use semi_wallet::client::postgres;
use semi_wallet::client::redis;
use semi_wallet::config;
use semi_wallet::repository::db::Repository;
use semi_wallet::service::coin::price_manager::PriceManager;
use semi_wallet::service::coin::price_storage::PriceStorage;
use semi_wallet::telemetry::{get_subscriber, init_subscriber};
use std::process;

use semi_wallet::handler::cmd::update_users_coins_amount::{self, UpdateUserCoinsCommand};
use semi_wallet::service::blockchain::service::Service as BlockchainService;
use semi_wallet::service::coin::service::Service as CoinService;
use semi_wallet::service::user_coin::service::Service as UserCoinService;
use semi_wallet::service::user_plan::service::Service as UserPlanService;

#[derive(Debug, Parser)]
#[command(name = "cli")]
#[command(about = "Semi-wallet CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    UpdateUsersCoinAmount(update_users_coins_amount::UpdateUserCoinsAmountArgs),
}

///cargo run --bin cli update-users-coin-amount  --user-id 1 --symbol "BTC"

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

    let http_client = reqwest::Client::builder().build();
    let http_client = match http_client {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("cannot create http_client due to err: {}", e);
            process::exit(1);
        }
    };
    let coin_service = CoinService::new(db_pool.clone(), repo.clone());
    let user_plan_service = UserPlanService::new(db_pool.clone(), repo.clone());

    let redis_client = redis::new_redis_client(cfg.redis.clone())
        .await
        .expect("cannot create redis client");

    let blockchain_service = BlockchainService::new(cfg, http_client);
    let blockchain_service = match blockchain_service {
        Ok(service) => service,
        Err(e) => {
            tracing::error!("cannot create blockchain service due to err: {}", e);
            process::exit(1);
        }
    };

    let price_storage = PriceStorage::new(redis_client);
    let price_manager = PriceManager::new(price_storage);

    let user_coin_service = UserCoinService::new(
        db_pool.clone(),
        repo.clone(),
        coin_service.clone(),
        user_plan_service.clone(),
        price_manager,
    );

    let args = Cli::parse();
    match args.command {
        Commands::UpdateUsersCoinAmount(args) => {
            let cmd = UpdateUserCoinsCommand::new(
                coin_service,
                user_coin_service,
                user_plan_service,
                blockchain_service,
            );
            cmd.run(args).await;
        }
    }
}
