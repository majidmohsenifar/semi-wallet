use std::collections::HashMap;

use crate::repository::models::Coin;

use super::price_storage::PriceStorage;
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

pub struct BinancePriceProvider<'a> {
    price_storage: PriceStorage,
    binance_pair_coin_symbol_map: HashMap<String, &'a str>,
    ws_url: String,
}

#[derive(Debug, Serialize)]
pub struct SubscribeRequest<'a> {
    pub id: u32,
    pub method: &'a str,
    pub params: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AvgPriceEvent {
    #[serde(rename = "e")]
    pub event: String,
    #[serde(rename = "E")]
    pub event_time: i64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "i")]
    pub average_price_interval: String,
    #[serde(rename = "w")]
    pub average_price: String,
    #[serde(rename = "T")]
    pub last_trade_time: i64,
}

impl<'a> BinancePriceProvider<'a> {
    pub fn new(price_storage: PriceStorage, coins: &'a [Coin], ws_url: String) -> Self {
        let map: HashMap<String, &str> = coins
            .iter()
            .map(|c| {
                {
                    (
                        c.price_pair_symbol
                            .as_ref()
                            .unwrap()
                            .as_str()
                            .replace('-', "")
                            .to_lowercase(),
                        c.symbol.as_ref(),
                    )
                }
            })
            .collect();
        BinancePriceProvider {
            price_storage,
            binance_pair_coin_symbol_map: map,
            ws_url,
        }
    }

    pub async fn run_update_prices(&self) {
        let (ws_stream, _) = connect_async(&self.ws_url).await.unwrap();
        let (mut writer, mut reader) = ws_stream.split();
        //maybe avgPrice is better, see the link https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams#average-price
        //let params = Vec::from(["btcusdt@avgPrice"]);
        //TODO: is using two iterator here necessary?
        let params: Vec<String> = self
            .binance_pair_coin_symbol_map
            .keys()
            .map(|k| format!("{}@avgPrice", k))
            .collect();

        let subscribe_request = SubscribeRequest {
            id: 1,
            method: "SUBSCRIBE",
            params,
        };
        let subscribe_msg = serde_json::to_string(&subscribe_request).unwrap();
        writer.send(Message::from(subscribe_msg)).await.unwrap();

        while let Some(message) = reader.try_next().await.unwrap() {
            if let Message::Text(text) = &message {
                let avg_price: AvgPriceEvent = serde_json::from_str(text).unwrap();
                let coin_symbol =
                    match self.get_coin_symbol_from_binance_symbol(avg_price.symbol.to_string()) {
                        Some(symbol) => symbol,
                        None => continue,
                    };
                self.price_storage
                    .set_price(coin_symbol, avg_price.average_price.parse().unwrap())
                    .await
                    .unwrap();
            } else if let Message::Ping(_) = &message {
                writer.send(Message::Pong(Vec::from([1]))).await.unwrap();
            } else if let Message::Close(_) = &message {
                break;
            }
        }
    }

    fn get_coin_symbol_from_binance_symbol(&self, binance_symbol: String) -> Option<&str> {
        self.binance_pair_coin_symbol_map
            .get(&binance_symbol.to_lowercase())
            .copied()
    }
}
