use std::collections::HashMap;

use alloy::primitives::{hex::ToHexExt, Address};
use bigdecimal::ToPrimitive;
use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::Duration;
use claims::assert_gt;
use semi_wallet::{
    client::postgres,
    handler::cmd::update_users_coins_amount::{UpdateUserCoinsAmountArgs, UpdateUserCoinsCommand},
    repository::{
        db::Repository, models::OrderStatus, order::CreateOrderArgs,
        user_plan::CreateUserPlanOrUpdateExpiresAtArgs,
    },
    service::blockchain::btc,
    service::blockchain::trx,
    service::coin::service::Service as CoinService,
    service::user_coin::service::Service as UserCoinService,
    service::user_plan::service::Service as UserPlanService,
    service::{
        blockchain::service::Service as BlockchainService, plan::service::PLAN_CODE_1_MONTH,
    },
};

use serde_json::json;
use solana_sdk::pubkey::Pubkey;
use wiremock::{
    matchers::{method, path},
    Mock, Request, ResponseTemplate,
};

use crate::helpers::{spawn_app, COINS};

//we have 2 users, one with active plan and all the coin and tokens need to be updated
//one with no active plan, which means none of his user_coins amount should be updated
#[tokio::test]
async fn update_users_coins_amount_without_args() {
    let app = spawn_app().await;
    let mut conn = app.db.acquire().await.unwrap();
    app.insert_coins().await;
    let plan = app
        .repo
        .get_plan_by_code(&app.db, PLAN_CODE_1_MONTH)
        .await
        .unwrap();

    let (_, user1) = app.get_jwt_token_and_user("test1@test.com").await;
    let old_order = app
        .repo
        .create_order(
            &mut conn,
            CreateOrderArgs {
                user_id: user1.id,
                plan_id: plan.id,
                total: plan.price.clone(),
                status: OrderStatus::Completed,
            },
        )
        .await
        .unwrap();

    app.repo
        .create_user_plan_or_update_expires_at(
            &mut conn,
            CreateUserPlanOrUpdateExpiresAtArgs {
                user_id: user1.id,
                plan_id: plan.id,
                order_id: old_order.id,
                days: 30,
            },
        )
        .await
        .unwrap();

    let eth_addr = Address::random();
    let sol_addr = Pubkey::new_unique();
    let tron_addr = "TENgyRvC2AzqcWZu4jEBdStA5UCpM2X8yA";

    app.create_user_coin(user1.id, "BTC", "BTC", "btc_addr_1")
        .await;
    app.create_user_coin(user1.id, "ETH", "ETH", &eth_addr.encode_hex())
        .await;
    app.create_user_coin(user1.id, "SOL", "SOL", &sol_addr.to_string())
        .await;
    app.create_user_coin(user1.id, "TRX", "TRX", tron_addr)
        .await;
    app.create_user_coin(user1.id, "USDT", "ETH", &eth_addr.encode_hex())
        .await;
    app.create_user_coin(user1.id, "USDT", "TRX", tron_addr)
        .await;

    //mocking btc node
    Mock::given(path(format!("{}/{}", btc::ADDRESS_URI, "btc_addr_1")))
        .and(method("GET"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(btc::GetAddressResponse {
                balance: "120000000".to_string(),
            }),
        )
        .mount(app.nodes.get("BTC").unwrap())
        .await;

    //mocking eth node
    Mock::given(path("/"))
        .and(method("POST"))
        .and(move |req: &Request| {
            let req_body: serde_json::Value = req.body_json().unwrap();
            let method = req_body.get("method").unwrap().as_str().unwrap();

            if method == "eth_getBalance" {
                let req_body: alloy::rpc::json_rpc::Request<Vec<String>> = req.body_json().unwrap();
                if req_body.params[0] != eth_addr.encode_hex_with_prefix() {
                    return false;
                }
                true
            } else {
                //TODO: I don't know how to check get token balance request
                true
            }
        })
        .respond_with(move |req: &Request| {
            let req_body: serde_json::Value = req.body_json().unwrap();
            let method = req_body.get("method").unwrap().as_str().unwrap();
            if method == "eth_getBalance" {
                ResponseTemplate::new(200).set_body_json(alloy::rpc::json_rpc::Response {
                    id: alloy::rpc::json_rpc::Id::Number(1),
                    payload: alloy::rpc::json_rpc::ResponsePayload::<_, Box<&str>>::Success(
                        format!("0x{:x}", 2_000_000_000_000_000_000i64),
                    ),
                })
            } else {
                ResponseTemplate::new(200).set_body_json({
                    json!({"id":1, "jsonrpc":"2.0", "result":"0x0000000000000000000000000000000000000000000000000000000077359400"})//2000 $
                })
            }
        })
        .mount(app.nodes.get("ETH").unwrap())
        .await;

    //mocking sol node
    Mock::given(path("/"))
        .and(method("POST"))
        .and(move |req: &Request| {
            let req_body: HashMap<String, serde_json::Value> = req.body_json().unwrap();
            let addr = match req_body.get("params") {
                None => return false,
                Some(val) => match val.as_array() {
                    None => return false,
                    Some(params) => &params[0],
                },
            };
            if addr.as_str().unwrap() != sol_addr.to_string() {
                return false;
            }
            true
        })
        .respond_with(ResponseTemplate::new(200).set_body_json({
            json!({
                "jsonrpc": "2.0",
                "result": { "context": { "slot": 1 }, "value": 2_000_000_000 },
                "id": 1
            })
        }))
        .mount(app.nodes.get("SOL").unwrap())
        .await;

    //mocking trx node
    Mock::given(path(trx::GET_ACCOUNT_URI))
        .and(method("POST"))
        .and(move |req: &Request| {
            let body: trx::GetAccountRequestBody = req.body_json().unwrap();
            if !body.visible {
                return false;
            }
            if body.address != tron_addr {
                return false;
            }
            true
        })
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
         "address": tron_addr,
        "balance": 2_000_000,
        })))
        .mount(app.nodes.get("TRX").unwrap())
        .await;

    //mocking USDT on trx node
    Mock::given(path(trx::TRIGGER_SMART_CONTRACT_URI))
        .and(method("POST"))
        .and(move |req: &Request| {
            let body: trx::TriggerConstantContractRequestBody = req.body_json().unwrap();
            if !body.visible {
                return false;
            }
            if body.function_selector != "balanceOf(address)" {
                return false;
            }
            if body.owner_address != tron_addr {
                return false;
            }
            if &body.contract_address
                != COINS
                    .get("USDT_TRX")
                    .unwrap()
                    .contract_address
                    .as_ref()
                    .unwrap()
            {
                return false;
            }
            let parameter = trx::get_hex_address(tron_addr).unwrap();
            if body.parameter != parameter {
                return false;
            }

            true
        })
        .respond_with({
            ResponseTemplate::new(200).set_body_json(trx::TriggerConstantContractResponseBody {
                result: trx::TriggerConstantContractResultResponse { result: true },
                constant_result: vec!["2000000000".to_string()],
            })
        })
        .mount(app.nodes.get("TRX").unwrap())
        .await;

    let (_, user2) = app.get_jwt_token_and_user("test2@test.com").await;
    app.create_user_coin(user2.id, "BTC", "BTC", "btc_addr_2")
        .await;
    app.create_user_coin(user2.id, "ETH", "ETH", "eth_addr_2")
        .await;

    let repo = Repository::default();
    let db_pool = postgres::new_pg_pool(&app.cfg.db.dsn).await;
    let http_client = reqwest::Client::builder().build().unwrap();
    let coin_service = CoinService::new(db_pool.clone(), repo.clone());
    let user_plan_service = UserPlanService::new(db_pool.clone(), repo.clone());
    let blockchain_service = BlockchainService::new(app.cfg, http_client).unwrap();
    let user_coin_service = UserCoinService::new(
        db_pool.clone(),
        repo.clone(),
        coin_service.clone(),
        user_plan_service.clone(),
    );

    let cmd = UpdateUserCoinsCommand::new(
        coin_service,
        user_coin_service,
        user_plan_service,
        blockchain_service,
    );
    cmd.run(UpdateUserCoinsAmountArgs {
        user_id: None,
        symbol: None,
    })
    .await;

    //checking users_coins be udpated
    let user1_coins = app
        .repo
        .get_user_coins_by_user_id(&app.db, user1.id)
        .await
        .unwrap();

    for uc in user1_coins {
        match (uc.symbol.as_str(), uc.network.as_str()) {
            ("BTC", "BTC") => {
                assert_eq!(uc.amount.unwrap().to_f64().unwrap(), 1.2);
                assert_gt!(
                    uc.amount_updated_at.unwrap().timestamp(),
                    (chrono::Utc::now() - Duration::minutes(5)).timestamp()
                );
            }
            ("ETH", "ETH") => {
                assert_eq!(uc.amount, BigDecimal::from_f64(2.0));
                assert_gt!(
                    uc.amount_updated_at.unwrap().timestamp(),
                    (chrono::Utc::now() - Duration::minutes(5)).timestamp()
                );
            }
            ("SOL", "SOL") => {
                assert_eq!(uc.amount, BigDecimal::from_u64(2));
                assert_gt!(
                    uc.amount_updated_at.unwrap().timestamp(),
                    (chrono::Utc::now() - Duration::minutes(5)).timestamp()
                );
            }
            ("TRX", "TRX") => {
                assert_eq!(uc.amount, BigDecimal::from_u64(2));
                assert_gt!(
                    uc.amount_updated_at.unwrap().timestamp(),
                    (chrono::Utc::now() - Duration::minutes(5)).timestamp()
                );
            }
            ("USDT", "ETH") => {
                assert_eq!(uc.amount, BigDecimal::from_u64(2000));
                assert_gt!(
                    uc.amount_updated_at.unwrap().timestamp(),
                    (chrono::Utc::now() - Duration::minutes(5)).timestamp()
                );
            }
            ("USDT", "TRX") => {
                assert_eq!(uc.amount, BigDecimal::from_u64(2000));
                assert_gt!(
                    uc.amount_updated_at.unwrap().timestamp(),
                    (chrono::Utc::now() - Duration::minutes(5)).timestamp()
                );
            }
            (_, _) => {
                panic!("we should not be here at all");
            }
        }
    }

    let user2_coins = app
        .repo
        .get_user_coins_by_user_id(&app.db, user2.id)
        .await
        .unwrap();

    for uc in user2_coins {
        if uc.amount.is_some() {
            //this should not happen
            panic!("user coin amount should be none");
        }
    }
}

#[tokio::test]
async fn update_users_coins_amount_with_args() {
    //TODO: impl later
}
