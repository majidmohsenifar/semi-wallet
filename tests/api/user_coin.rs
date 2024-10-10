use std::collections::HashMap;

use claim::assert_gt;
use semi_wallet::{
    handler::response::{ApiError, ApiResponse},
    repository::user_coin::CreateUserCoinArgs,
    service::user_coin::service::UserCoin,
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn get_user_coins_without_token() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/api/v1/user-coins", app.address))
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(
        401,
        response.status().as_u16(),
        "the api did not fail with 401 unauthorized"
    );
}

#[tokio::test]
async fn get_user_coins_successful() {
    let app = spawn_app().await;
    let (token, user) = app.get_jwt_token("test@test.test").await;

    app.insert_coins().await;

    app.repo
        .create_user_coin(
            &app.db,
            CreateUserCoinArgs {
                user_id: user.id,
                coin_id: 1,
                symbol: "BTC".to_string(),
                network: "BTC".to_string(),
                address: "btc_addr".to_string(),
            },
        )
        .await
        .unwrap();

    app.repo
        .create_user_coin(
            &app.db,
            CreateUserCoinArgs {
                user_id: user.id,
                coin_id: 2,
                symbol: "ETH".to_string(),
                network: "ETH".to_string(),
                address: "eth_addr".to_string(),
            },
        )
        .await
        .unwrap();

    app.repo
        .create_user_coin(
            &app.db,
            CreateUserCoinArgs {
                user_id: user.id,
                coin_id: 3,
                symbol: "USDT".to_string(),
                network: "ETH".to_string(),
                address: "usdt_eth_addr".to_string(),
            },
        )
        .await
        .unwrap();

    app.repo
        .create_user_coin(
            &app.db,
            CreateUserCoinArgs {
                user_id: user.id,
                coin_id: 4,
                symbol: "USDT".to_string(),
                network: "TRX".to_string(),
                address: "usdt_trx_addr".to_string(),
            },
        )
        .await
        .unwrap();

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/api/v1/user-coins", app.address))
        .bearer_auth(token)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(
        200,
        response.status().as_u16(),
        "the api did not fail with 401 unauthorized"
    );

    let bytes = response.bytes().await.unwrap();
    let res: ApiResponse<'_, Vec<UserCoin>> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(res.message, "");
    let data = res.data.unwrap();
    assert_eq!(data.len(), 4);

    let uc1 = data.first().unwrap();
    assert_gt!(uc1.id, 0);
    assert_gt!(uc1.coin_id, 0);
    assert_eq!(uc1.address, "btc_addr".to_string());
    assert_eq!(uc1.symbol, "BTC".to_string());
    assert_eq!(uc1.network, "BTC".to_string());

    let uc2 = data.get(1).unwrap();
    assert_gt!(uc2.id, 0);
    assert_gt!(uc2.coin_id, 0);
    assert_eq!(uc2.address, "eth_addr".to_string());
    assert_eq!(uc2.symbol, "ETH".to_string());
    assert_eq!(uc2.network, "ETH".to_string());

    let uc3 = data.get(2).unwrap();
    assert_gt!(uc3.id, 0);
    assert_gt!(uc3.coin_id, 0);
    assert_eq!(uc3.address, "usdt_eth_addr".to_string());
    assert_eq!(uc3.symbol, "USDT".to_string());
    assert_eq!(uc3.network, "ETH".to_string());

    let uc4 = data.last().unwrap();
    assert_gt!(uc4.id, 0);
    assert_gt!(uc4.coin_id, 0);
    assert_eq!(uc4.address, "usdt_trx_addr".to_string());
    assert_eq!(uc4.symbol, "USDT".to_string());
    assert_eq!(uc4.network, "TRX".to_string());
}

#[tokio::test]
async fn create_user_coin_without_token() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = HashMap::from([
        ("address", "btc_addr"),
        ("symbol", "BTC"),
        ("network", "BTC"),
    ]);
    let response = client
        .post(&format!("{}/api/v1/user-coins/create", app.address))
        .json(&body)
        .send()
        .await
        .expect("failed to call the api");
    assert_eq!(
        401,
        response.status().as_u16(),
        "the api did not fail with 401 Unauthorized",
    );
}

#[tokio::test]
async fn create_user_coin_invalid_inputs() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let (token, _) = app.get_jwt_token("test@test.com").await;
    let test_cases: Vec<(HashMap<&str, &str>, &str)> = vec![
        (HashMap::new(), "without address"),
        (HashMap::from([("network", "BTC")]), "empty address"),
        (
            HashMap::from([("address", "btc_addr"), ("network", "BTC")]),
            "without symbol",
        ),
        (
            HashMap::from([("address", "btc_addr"), ("symbol", ""), ("network", "BTC")]),
            "empty symbol",
        ),
    ];

    for (body, scenario) in test_cases {
        let response = client
            .post(&format!("{}/api/v1/user-coins/create", app.address))
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .expect("failed to call the api");
        assert_eq!(
            400,
            response.status().as_u16(),
            "the api did not fail with 400 Bad Request when the payload has the problem {}",
            scenario
        );
    }
}

#[tokio::test]
async fn create_user_coin_coin_not_found() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let (token, _) = app.get_jwt_token("test@test.com").await;
    app.insert_coins().await;
    let body = HashMap::from([
        ("address", "btc_addr"),
        ("symbol", "not_found"),
        ("network", "BTC"),
    ]);
    let response = client
        .post(&format!("{}/api/v1/user-coins/create", app.address))
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .expect("failed to call the api");
    assert_eq!(
        404,
        response.status().as_u16(),
        "the api did not fail with 401 Unauthorized",
    );
    let bytes = response.bytes().await.unwrap();
    let res: ApiError<'_> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(res.message, "coin or network not found");
}

#[tokio::test]
async fn create_user_coin_successful() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let (token, _) = app.get_jwt_token("test@test.com").await;
    app.insert_coins().await;
    let body = HashMap::from([
        ("address", "btc_addr"),
        ("symbol", "BTC"),
        ("network", "BTC"),
    ]);
    let response = client
        .post(&format!("{}/api/v1/user-coins/create", app.address))
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .expect("failed to call the api");
    assert_eq!(
        200,
        response.status().as_u16(),
        "the api status code is not 200 Ok",
    );
    let bytes = response.bytes().await.unwrap();
    let res: ApiResponse<'_, UserCoin> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(res.message, "");
    let data = res.data.unwrap();
    assert_gt!(data.id, 0);
    assert_eq!(data.coin_id, 1);
    assert_eq!(data.address, "btc_addr");
    assert_eq!(data.symbol, "BTC");
    assert_eq!(data.network, "BTC");
}

#[tokio::test]
async fn create_user_coin_network_not_set_successful() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let (token, _) = app.get_jwt_token("test@test.com").await;
    app.insert_coins().await;
    let body = HashMap::from([("address", "btc_addr"), ("symbol", "BTC")]);
    let response = client
        .post(&format!("{}/api/v1/user-coins/create", app.address))
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .expect("failed to call the api");
    assert_eq!(
        200,
        response.status().as_u16(),
        "the api status code is not 200 Ok",
    );
    let bytes = response.bytes().await.unwrap();
    let res: ApiResponse<'_, UserCoin> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(res.message, "");
    let data = res.data.unwrap();
    assert_gt!(data.id, 0);
    assert_eq!(data.coin_id, 1);
    assert_eq!(data.address, "btc_addr");
    assert_eq!(data.symbol, "BTC");
    assert_eq!(data.network, "BTC");
}

#[tokio::test]
async fn create_user_coin_empty_network_set_successful() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let (token, _) = app.get_jwt_token("test@test.com").await;
    app.insert_coins().await;
    let body = HashMap::from([("address", "btc_addr"), ("symbol", "BTC"), ("network", " ")]);
    let response = client
        .post(&format!("{}/api/v1/user-coins/create", app.address))
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .expect("failed to call the api");
    assert_eq!(
        200,
        response.status().as_u16(),
        "the api status code is not 200 Ok",
    );
    let bytes = response.bytes().await.unwrap();
    let res: ApiResponse<'_, UserCoin> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(res.message, "");
    let data = res.data.unwrap();
    assert_gt!(data.id, 0);
    assert_eq!(data.coin_id, 1);
    assert_eq!(data.address, "btc_addr");
    assert_eq!(data.symbol, "BTC");
    assert_eq!(data.network, "BTC");
}

#[tokio::test]
async fn create_user_coin_with_network_set_successful() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let (token, _) = app.get_jwt_token("test@test.com").await;
    app.insert_coins().await;
    let body = HashMap::from([
        ("address", "usdt_addr"),
        ("symbol", "USDT"),
        ("network", "ETH"),
    ]);
    let response = client
        .post(&format!("{}/api/v1/user-coins/create", app.address))
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .expect("failed to call the api");
    assert_eq!(
        200,
        response.status().as_u16(),
        "the api status code is not 200 Ok",
    );
    let bytes = response.bytes().await.unwrap();
    let res: ApiResponse<'_, UserCoin> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(res.message, "");
    let data = res.data.unwrap();
    assert_gt!(data.id, 0);
    assert_eq!(data.coin_id, 3);
    assert_eq!(data.address, "usdt_addr");
    assert_eq!(data.symbol, "USDT");
    assert_eq!(data.network, "ETH");
}
