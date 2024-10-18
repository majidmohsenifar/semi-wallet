use clap::{Arg, ArgAction, Command};

pub fn update_users_coins_amount_command() -> Command {
    Command::new("update-users-coins-amount")
        .about("getting amount of user coins ")
        .arg(
            Arg::new("user_id")
                .long("user_id")
                .help("specific user to update coin amount")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("symbol")
                .long("symbol")
                .help("specific coin symbol to update coin amount")
                .action(ArgAction::Set),
        )
}
