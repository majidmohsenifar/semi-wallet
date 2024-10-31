use semi_wallet::{
    client::postgres,
    config,
    handler::cmd::update_users_coins_amount::{UpdateUserCoinsAmountArgs, UpdateUserCoinsCommand},
    repository::db::Repository,
    service::blockchain::service::Service as BlockchainService,
    service::coin::service::Service as CoinService,
    service::user_coin::service::Service as UserCoinService,
    service::user_plan::service::Service as UserPlanService,
};

use wiremock::{
    matchers::{method, path},
    Mock, Request, ResponseTemplate,
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn update_users_coins_amount_without_args() {
    //let app = spawn_app().await;

    //Mock::given(path("/filan"))
    //.and(method("POST"))
    //.and(move |_request: &Request| true)
    //.respond_with(ResponseTemplate::new(200).set_body_json("did")) //
    //.mount(app.nodes.get("BTC").unwrap())
    //.await;

    //let repo = Repository::default();
    //let db_pool = postgres::new_pg_pool(&app.cfg.db.dsn).await;
    //let http_client = reqwest::Client::builder().build().unwrap();
    //let coin_service = CoinService::new(db_pool.clone(), repo.clone());
    //let user_plan_service = UserPlanService::new(db_pool.clone(), repo.clone());
    //let blockchain_service = BlockchainService::new(app.cfg, http_client).unwrap();
    //let user_coin_service = UserCoinService::new(
    //db_pool.clone(),
    //repo.clone(),
    //coin_service.clone(),
    //user_plan_service.clone(),
    //);

    //let cmd = UpdateUserCoinsCommand::new(
    //coin_service,
    //user_coin_service,
    //user_plan_service,
    //blockchain_service,
    //);
    //cmd.run(UpdateUserCoinsAmountArgs {
    //user_id: None,
    //symbol: None,
    //})
    //.await;
}
