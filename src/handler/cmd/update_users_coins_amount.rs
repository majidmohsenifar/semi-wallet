use clap::Args;

//pub const UPDATE_USERS_COINS_AMOUNT_COMMAND: &str = "update-users-coins-amount";

#[derive(Debug, Args)]
#[command(flatten_help = true)]
pub struct UpdateUserCoinsAmountArgs {
    #[arg(short, long)]
    user_id: Option<i64>,
    #[arg(short, long)]
    symbol: Option<String>,
}
pub fn update_users_coins_amount_command(args: UpdateUserCoinsAmountArgs) {
    println!("user_id in command {}", args.user_id.unwrap());
    println!("symbol in command {}", args.symbol.unwrap());
}
