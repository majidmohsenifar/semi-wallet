use semi_wallet::service::coin::{
    price_manager::{PriceManager, PRICE_PROVIDER_BINANCE},
    price_storage::PriceStorage,
};

use serde_json::json;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use ws_mock::{matchers::StringExact, ws_mock_server::WsMock};

use crate::helpers::spawn_app;

#[tokio::test]
async fn update_coin_prices_using_binance_ws() {
    let app = spawn_app().await;
    app.insert_coins().await;
    let coins = app
        .repo
        .get_not_null_price_pair_symbol_coins(&app.db)
        .await
        .expect("cannot get coins");
    if coins.is_empty() {
        panic!("coins is empty");
    }

    //running binance ws server
    let (mpsc_send, mpsc_recv) = mpsc::channel::<Message>(32);
    let subscribe_res = json!({
    "result": null,
    "id": 1
    });
    WsMock::new()
        .matcher(StringExact::new(r#"{"id":1,"method":"SUBSCRIBE","params":["btcusdt@avgPrice","ethusdt@avgPrice","solusdt@avgPrice","trxusdt@avgPrice"]}"#))
        .respond_with(Message::Text(subscribe_res.to_string()))
        .expect(1)
        .forward_from_channel(mpsc_recv)
        .mount(&app.binance_ws_server)
        .await;

    let price_storage = PriceStorage::new(app.redis_client);
    let price_manager = PriceManager::new(price_storage);
    let price_manager2 = price_manager.clone();
    let binance_cfg = app.cfg.binance.clone();

    tokio::spawn(async move {
        price_manager2
            .clone()
            .run_update_prices(PRICE_PROVIDER_BINANCE, &coins, binance_cfg)
            .await;
    });
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let btc_data = json!({
          "e": "avgPrice",          // Event type
          "E": 1693907033,          // Event time
          "s": "BTCUSDT",           // Symbol
          "i": "5m",                // Average price interval
          "w": "100000.00000000",   // Average price
          "T": 1693907032           // Last trade time
    });
    let eth_data = json!({
          "e": "avgPrice",
          "E": 1693907033,
          "s": "ETHUSDT",
          "i": "5m",
          "w": "4000.00000000",
          "T": 1693907032
    });
    let sol_data = json!({
          "e": "avgPrice",
          "E": 1693907033,
          "s": "SOLUSDT",
          "i": "5m",
          "w": "250.00000000",
          "T": 1693907032
    });
    let trx_data = json!({
          "e": "avgPrice",
          "E": 1693907033,
          "s": "TRXUSDT",
          "i": "5m",
          "w": "0.4",
          "T": 1693907032
    });
    mpsc_send
        .send(Message::Text(btc_data.to_string()))
        .await
        .unwrap();
    mpsc_send
        .send(Message::Text(eth_data.to_string()))
        .await
        .unwrap();
    mpsc_send
        .send(Message::Text(sol_data.to_string()))
        .await
        .unwrap();
    mpsc_send
        .send(Message::Text(trx_data.to_string()))
        .await
        .unwrap();

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    //assert results
    let prices = price_manager
        .get_prices_for_coins(vec!["BTC", "ETH", "SOL", "TRX"])
        .await
        .unwrap();

    let btc_price_data = prices.get("BTC").unwrap();
    assert_eq!(btc_price_data.price, 100_000f64);
    let eth_price_data = prices.get("ETH").unwrap();
    assert_eq!(eth_price_data.price, 4_000f64);
    let sol_price_data = prices.get("SOL").unwrap();
    assert_eq!(sol_price_data.price, 250f64);
    let trx_price_data = prices.get("TRX").unwrap();
    assert_eq!(trx_price_data.price, 0.4);

    app.binance_ws_server.verify().await;
}
