use std::collections::HashMap;
use std::convert::From;

use bigdecimal::{BigDecimal, FromPrimitive};
use claim::{assert_gt, assert_none};
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
    let (token, user) = app.get_jwt_token_and_user("test@test.test").await;

    app.insert_coins().await;

    let uc1 = app
        .repo
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
        .update_user_coin_amount(&app.db, uc1, BigDecimal::from_f64(2.18).unwrap())
        .await
        .unwrap();

    let uc2 = app
        .repo
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
        .update_user_coin_amount(&app.db, uc2, BigDecimal::from_f64(0.0002).unwrap())
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
    assert_eq!(uc1.amount.unwrap(), 2.18);
    assert_gt!(uc1.amount_updated_at.unwrap(), 0);

    let uc2 = data.get(1).unwrap();
    assert_gt!(uc2.id, 0);
    assert_gt!(uc2.coin_id, 0);
    assert_eq!(uc2.address, "eth_addr".to_string());
    assert_eq!(uc2.symbol, "ETH".to_string());
    assert_eq!(uc2.network, "ETH".to_string());
    assert_eq!(uc2.amount.unwrap(), 0.0002);
    assert_gt!(uc2.amount_updated_at.unwrap(), 0);

    let uc3 = data.get(2).unwrap();
    assert_gt!(uc3.id, 0);
    assert_gt!(uc3.coin_id, 0);
    assert_eq!(uc3.address, "usdt_eth_addr".to_string());
    assert_eq!(uc3.symbol, "USDT".to_string());
    assert_eq!(uc3.network, "ETH".to_string());
    assert_none!(uc3.amount);
    assert_none!(uc3.amount_updated_at);

    let uc4 = data.last().unwrap();
    assert_gt!(uc4.id, 0);
    assert_gt!(uc4.coin_id, 0);
    assert_eq!(uc4.address, "usdt_trx_addr".to_string());
    assert_eq!(uc4.symbol, "USDT".to_string());
    assert_eq!(uc4.network, "TRX".to_string());
    assert_none!(uc4.amount);
    assert_none!(uc4.amount_updated_at);
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
    let (token, _) = app.get_jwt_token_and_user("test@test.com").await;
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
    let (token, _) = app.get_jwt_token_and_user("test@test.com").await;
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
    let (token, _) = app.get_jwt_token_and_user("test@test.com").await;
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
    let (token, _) = app.get_jwt_token_and_user("test@test.com").await;
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
    let (token, _) = app.get_jwt_token_and_user("test@test.com").await;
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
    let (token, _) = app.get_jwt_token_and_user("test@test.com").await;
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

#[tokio::test]
async fn delete_user_coin_invalid_inputs() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let (token, _) = app.get_jwt_token_and_user("test@test.com").await;
    app.insert_coins().await;

    let test_cases: Vec<(HashMap<&str, &str>, &str)> = vec![
        (HashMap::new(), "id not set"),
        (HashMap::from([("id", "wrong")]), "id is not int"),
    ];

    for (body, scenario) in test_cases {
        let response = client
            .delete(&format!("{}/api/v1/user-coins/delete", app.address))
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .expect("failed to call the api");
        assert_eq!(
            400,
            response.status().as_u16(),
            "the api status code is not 400 Bad Request for the case {}",
            scenario
        );
    }
}

#[tokio::test]
async fn delete_user_coin_user_coin_not_found() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let (token, _) = app.get_jwt_token_and_user("test@test.com").await;
    app.insert_coins().await;
    let body = HashMap::from([("id", 12)]);
    let response = client
        .delete(&format!("{}/api/v1/user-coins/delete", app.address))
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .expect("failed to call the api");
    assert_eq!(
        404,
        response.status().as_u16(),
        "the api status code is not 404",
    );
    let bytes = response.bytes().await.unwrap();
    let res: ApiError<'_> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(res.message, "user coin not found");
}

#[tokio::test]
async fn delete_user_coin_user_coin_does_not_belong_to_user() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let (token, _) = app.get_jwt_token_and_user("test@test.com").await;
    let (_, other_user) = app.get_jwt_token_and_user("test2@test.com").await;
    app.insert_coins().await;

    let user_coin_id = app
        .repo
        .create_user_coin(
            &app.db,
            CreateUserCoinArgs {
                user_id: other_user.id,
                coin_id: 1,
                symbol: "BTC".to_string(),
                network: "BTC".to_string(),
                address: "btc_addr".to_string(),
            },
        )
        .await
        .unwrap();

    let body = HashMap::from([("id", user_coin_id)]);
    let response = client
        .delete(&format!("{}/api/v1/user-coins/delete", app.address))
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .expect("failed to call the api");
    assert_eq!(
        404,
        response.status().as_u16(),
        "the api status code is not 404",
    );
    let bytes = response.bytes().await.unwrap();
    let res: ApiError<'_> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(res.message, "user coin not found");
}

#[tokio::test]
async fn delete_user_coin_successful() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let (token, user) = app.get_jwt_token_and_user("test@test.com").await;
    app.insert_coins().await;

    let user_coin_id = app
        .repo
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

    let body = HashMap::from([("id", user_coin_id)]);
    let response = client
        .delete(&format!("{}/api/v1/user-coins/delete", app.address))
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
    let res: ApiResponse<'_, Option<()>> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(res.message, "");

    let user_coins = app
        .repo
        .get_user_coins_by_user_id(&app.db, user.id)
        .await
        .unwrap();
    assert_eq!(user_coins.len(), 0);
}

#[tokio::test]
async fn update_user_coin_address_invalid_inputs() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let (token, _) = app.get_jwt_token_and_user("test@test.com").await;
    app.insert_coins().await;

    let test_cases: Vec<(HashMap<&str, serde_json::Value>, &str)> = vec![
        (
            HashMap::from([("address", serde_json::Value::String("addr".to_string()))]),
            "id is required",
        ),
        (
            HashMap::from([("id", serde_json::Value::String("wrong".to_string()))]),
            "id is not i64",
        ),
        (
            HashMap::from([("id", serde_json::Value::Number(serde_json::Number::from(2)))]),
            "address is required",
        ),
        (
            HashMap::from([
                ("id", serde_json::Value::Number(serde_json::Number::from(2))),
                (
                    "address",
                    serde_json::Value::Number(serde_json::Number::from(5)),
                ),
            ]),
            "address is not string",
        ),
    ];

    for (body, msg) in test_cases {
        let response = client
            .patch(&format!("{}/api/v1/user-coins/update-address", app.address))
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .expect("failed to call the api");
        assert_eq!(
            400,
            response.status().as_u16(),
            "the api status code is not 400 Bad Request",
        );

        let bytes = response.bytes().await.unwrap();
        let res: ApiError<'_> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(res.message, msg);
    }
}

#[tokio::test]
async fn update_user_coin_address_not_found() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let (token, _) = app.get_jwt_token_and_user("test@test.com").await;
    app.insert_coins().await;
    let body = HashMap::from([
        ("id", serde_json::Value::Number(serde_json::Number::from(2))),
        ("address", serde_json::Value::String("addr".to_string())),
    ]);

    let response = client
        .patch(&format!("{}/api/v1/user-coins/update-address", app.address))
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .expect("failed to call the api");
    assert_eq!(
        404,
        response.status().as_u16(),
        "the api status code is not 404",
    );

    let bytes = response.bytes().await.unwrap();
    let res: ApiError<'_> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(res.message, "user coin not found");
}

#[tokio::test]
async fn update_user_coin_does_not_belong_to_user() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let (token, _) = app.get_jwt_token_and_user("test@test.com").await;
    let (_, other_user) = app.get_jwt_token_and_user("test2@test.com").await;
    app.insert_coins().await;

    let user_coin_id = app
        .repo
        .create_user_coin(
            &app.db,
            CreateUserCoinArgs {
                user_id: other_user.id,
                coin_id: 1,
                symbol: "BTC".to_string(),
                network: "BTC".to_string(),
                address: "btc_addr".to_string(),
            },
        )
        .await
        .unwrap();

    let body = HashMap::from([
        (
            "id",
            serde_json::Value::Number(serde_json::Number::from(user_coin_id)),
        ),
        ("address", serde_json::Value::String("addr".to_string())),
    ]);

    let response = client
        .patch(&format!("{}/api/v1/user-coins/update-address", app.address))
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .expect("failed to call the api");
    assert_eq!(
        404,
        response.status().as_u16(),
        "the api status code is not 404",
    );

    let bytes = response.bytes().await.unwrap();
    let res: ApiError<'_> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(res.message, "user coin not found");
}

#[tokio::test]
async fn update_user_coin_successful() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let (token, user) = app.get_jwt_token_and_user("test@test.com").await;
    app.insert_coins().await;

    let user_coin_id = app
        .repo
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

    let body = HashMap::from([
        (
            "id",
            serde_json::Value::Number(serde_json::Number::from(user_coin_id)),
        ),
        (
            "address",
            serde_json::Value::String("updated_btc_addr".to_string()),
        ),
    ]);

    let response = client
        .patch(&format!("{}/api/v1/user-coins/update-address", app.address))
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .expect("failed to call the api");
    assert_eq!(
        200,
        response.status().as_u16(),
        "the api status code is not 404",
    );

    let bytes = response.bytes().await.unwrap();
    let res: ApiResponse<'_, Option<()>> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(res.message, "");

    let user_coins = app
        .repo
        .get_user_coins_by_user_id(&app.db, user.id)
        .await
        .unwrap();

    let uc = user_coins.first().unwrap();
    assert_eq!(uc.user_id, user.id);
    assert_eq!(uc.coin_id, 1);
    assert_eq!(uc.address, "updated_btc_addr");
    assert_eq!(uc.symbol, "BTC");
    assert_eq!(uc.network, "BTC");
}
