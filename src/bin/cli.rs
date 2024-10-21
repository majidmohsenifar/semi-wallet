use semi_wallet::handler::cmd;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "cli")]
#[command(about = "Semi-wallet CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    UpdateUsersCoinAmount(cmd::update_users_coins_amount::UpdateUserCoinsAmountArgs),
}

///cargo run --bin cli update-users-coin-amount  --user-id 1 --symbol "BTC"

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::UpdateUsersCoinAmount(args) => {
            cmd::update_users_coins_amount::update_users_coins_amount_command(args);
        }
    }
}
