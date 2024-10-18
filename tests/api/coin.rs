use semi_wallet::handler::api::response::ApiResponse;
use semi_wallet::service::coin::service::Coin;

use crate::helpers::spawn_app;

#[tokio::test]
async fn get_coins_list_successful() {
    let app = spawn_app().await;

    app.insert_coins().await;
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
    let data = res.data.unwrap();
    assert_eq!(data.len(), 4);
    let btc = data.first().unwrap();
    assert_eq!(btc.symbol, "BTC");
    assert_eq!(btc.name, "Bitcoin");
    assert_eq!(btc.logo, "btc.png");
    assert_eq!(btc.network, "BTC");
    assert_eq!(btc.decimals, 8);
    assert_eq!(btc.description, "Bitcoin is the best");

    let eth = data.get(1).unwrap();
    assert_eq!(eth.symbol, "ETH");
    assert_eq!(eth.name, "Ethereum");
    assert_eq!(eth.logo, "eth.png");
    assert_eq!(eth.network, "ETH");
    assert_eq!(eth.decimals, 18);
    assert_eq!(eth.description, "Ethereum is the second best");

    let tether_eth = data.get(2).unwrap();
    assert_eq!(tether_eth.symbol, "USDT");
    assert_eq!(tether_eth.name, "Tether");
    assert_eq!(tether_eth.logo, "usdt.png");
    assert_eq!(tether_eth.network, "ETH");
    assert_eq!(tether_eth.decimals, 18);
    assert_eq!(tether_eth.description, "Tether is the third best");

    let tether_trx = data.last().unwrap();

    assert_eq!(tether_trx.symbol, "USDT");
    assert_eq!(tether_trx.name, "Tether");
    assert_eq!(tether_trx.logo, "usdt_trx.png");
    assert_eq!(tether_trx.network, "TRX");
    assert_eq!(tether_trx.decimals, 18);
    assert_eq!(tether_trx.description, "Tether is the third best");
}
