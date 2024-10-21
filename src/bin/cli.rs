use clap::{Parser, Subcommand};
use semi_wallet::client::postgres;
use semi_wallet::config;
use semi_wallet::repository::db::Repository;
use semi_wallet::telemetry::{get_subscriber, init_subscriber};

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
    let cfg = config::Settings::new().expect("cannot parse configuration");
    let subscriber = get_subscriber("semi-wallet-cli".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    let repo = Repository::new();
    let db_pool = postgres::new_pg_pool(&cfg.db.dsn).await;
    let coin_service = CoinService::new(db_pool.clone(), repo.clone());
    let user_plan_service = UserPlanService::new(db_pool.clone(), repo.clone());
    let blockchain_service = BlockchainService::new(cfg);
    let user_coin_service = UserCoinService::new(
        db_pool.clone(),
        repo.clone(),
        coin_service.clone(),
        user_plan_service.clone(),
    );

    let args = Cli::parse();
    match args.command {
        Commands::UpdateUsersCoinAmount(args) => {
            let cmd = UpdateUserCoinsCommand::new(
                user_coin_service,
                user_plan_service,
                blockchain_service,
            );
            cmd.run(args).await;
        }
    }
}
