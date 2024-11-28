use futures_util::{SinkExt, StreamExt, TryStreamExt};
use serde_json::json;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() {
    //TODO: move this url to config
    let url = "wss://testnet.binance.vision/ws";
    let (ws_stream, _) = connect_async(url).await.unwrap();
    let (mut writer, mut reader) = ws_stream.split();
    //maybe avgPrice is better, see the link https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams#average-price
    //TODO: we should create params dynamically related to our coins
    let params = Vec::from(["btcusdt@ticker"]);
    //TODO: convert this json to struct and serialize that
    let subscribe_request = json!({
    "id":1,
    "method":"SUBSCRIBE",
    "params":params,
    });
    writer
        .send(Message::from(subscribe_request.to_string()))
        .await
        .unwrap();

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
