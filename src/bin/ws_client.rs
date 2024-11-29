use std::process;

use futures_util::{SinkExt, StreamExt, TryStreamExt};
use semi_wallet::config;
use serde::Serialize;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Serialize)]
pub struct SubscribeRequest<'a> {
    id: u32,
    method: &'a str,
    params: Vec<&'a str>,
}

#[tokio::main]
async fn main() {
    let cfg = config::Settings::new();
    let cfg = match cfg {
        Ok(cfg) => cfg,
        Err(e) => {
            tracing::error!("cannot create configs to err: {}", e);
            process::exit(1);
        }
    };

    let url = cfg.binance.ws_url.as_str();
    let (ws_stream, _) = connect_async(url).await.unwrap();
    let (mut writer, mut reader) = ws_stream.split();
    //maybe avgPrice is better, see the link https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams#average-price
    //TODO: we should create params dynamically related to our coins
    let params = Vec::from(["btcusdt@avgPrice"]);
    let subscribe_request = SubscribeRequest {
        id: 1,
        method: "SUBSCRIBE",
        params,
    };
    let subscribe_msg = serde_json::to_string(&subscribe_request).unwrap();
    writer.send(Message::from(subscribe_msg)).await.unwrap();

    while let Some(message) = reader.try_next().await.unwrap() {
        if let Message::Text(text) = &message {
            //TODO: handle the ticker here
            println!("{text}");
        } else if let Message::Ping(_) = &message {
            writer.send(Message::Pong(Vec::from([1]))).await.unwrap();
        } else if let Message::Close(_) = &message {
            break;
        }
    }
}
