use semi_wallet::handler::response::ApiResponse;
use semi_wallet::repository::coin::CreateCoinArgs;
use semi_wallet::service::coin::service::Coin;

use crate::helpers::spawn_app;

#[tokio::test]
async fn get_coin_lists_successful() {
    let app = spawn_app().await;

    let btc_coin = app
        .repo
        .create_coin(
            &app.db,
            CreateCoinArgs {
                symbol: "BTC".to_string(),
                name: "Bitcoin".to_string(),
                network: "BTC".to_string(),
                logo: "btc.png".to_string(),
                decimals: 8,
                description: "Bitcoin is the best".to_string(),
            },
        )
        .await
        .unwrap();
    println!("{:?}", btc_coin);

    let eth_coin = app
        .repo
        .create_coin(
            &app.db,
            CreateCoinArgs {
                symbol: "ETH".to_string(),
                name: "Ethereum".to_string(),
                network: "ETH".to_string(),
                logo: "eth.png".to_string(),
                decimals: 18,
                description: "Ethereum is the second best".to_string(),
            },
        )
        .await
        .unwrap();

    let usdt_eth_coin = app
        .repo
        .create_coin(
            &app.db,
            CreateCoinArgs {
                symbol: "USDT".to_string(),
                name: "Tether".to_string(),
                network: "ETH".to_string(),
                logo: "usdt.png".to_string(),
                decimals: 18,
                description: "Tether is the third best".to_string(),
            },
        )
        .await
        .unwrap();

    let usdt_trx_coin = app
        .repo
        .create_coin(
            &app.db,
            CreateCoinArgs {
                symbol: "USDT".to_string(),
                name: "Tether".to_string(),
                network: "TRX".to_string(),
                logo: "usdt_trx.png".to_string(),
                decimals: 18,
                description: "Tether is the third best".to_string(),
            },
        )
        .await
        .unwrap();

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/api/v1/coins", app.address))
        .send()
        .await
        .expect("failed to execute request");
    assert_eq!(200, response.status().as_u16(),);
    let bytes = response.bytes().await.unwrap();
    let res: ApiResponse<'_, Vec<Coin>> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(res.message, "");
    //TODO: validate the list itself
    let data = res.data.unwrap();
    assert_eq!(data.len(), 4);
    let btc = data.first().unwrap();
    assert_eq!(btc.id, btc_coin.id);
    assert_eq!(btc.symbol, "BTC");
    assert_eq!(btc.name, "Bitcoin");
    assert_eq!(btc.logo, "btc.png");
    assert_eq!(btc.network, "BTC");
    assert_eq!(btc.decimals, 8);
    assert_eq!(btc.description, "Bitcoin is the best");

    let eth = data.get(1).unwrap();
    assert_eq!(eth.id, eth_coin.id);
    assert_eq!(eth.symbol, "ETH");
    assert_eq!(eth.name, "Ethereum");
    assert_eq!(eth.logo, "eth.png");
    assert_eq!(eth.network, "ETH");
    assert_eq!(eth.decimals, 18);
    assert_eq!(eth.description, "Ethereum is the second best");

    let tether_eth = data.get(2).unwrap();
    assert_eq!(tether_eth.id, usdt_eth_coin.id);
    assert_eq!(tether_eth.symbol, "USDT");
    assert_eq!(tether_eth.name, "Tether");
    assert_eq!(tether_eth.logo, "usdt.png");
    assert_eq!(tether_eth.network, "ETH");
    assert_eq!(tether_eth.decimals, 18);
    assert_eq!(tether_eth.description, "Tether is the third best");

    let tether_trx = data.last().unwrap();

    assert_eq!(tether_trx.id, usdt_trx_coin.id);
    assert_eq!(tether_trx.symbol, "USDT");
    assert_eq!(tether_trx.name, "Tether");
    assert_eq!(tether_trx.logo, "usdt_trx.png");
    assert_eq!(tether_trx.network, "TRX");
    assert_eq!(tether_trx.decimals, 18);
    assert_eq!(tether_trx.description, "Tether is the third best");
}
