use semi_wallet::handler::cmd;

#[tokio::main]
async fn main() {
    let cli_app = cmd::command::command()
        .subcommand(cmd::update_users_coins_amount::update_users_coins_amount_command());
    //TODO: we should use try_get_matches
    let _matches = cli_app.get_matches();

    //let command = matches.subcommand().map_or_else(||{

    //}, f);

    //let res = matches.subcommand().map_or_else(
    //|| cmd::default::run(&matches),
    //|tup| match tup {
    //("validate", subcommand_matches) => cmd::validate::run(&matches, subcommand_matches),
    //_ => unreachable!(),
    //},
    //);

    //cmd::result_exit(res);
}
