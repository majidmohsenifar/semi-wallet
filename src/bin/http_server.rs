use semi_wallet::config;
use semi_wallet::telemetry::{get_subscriber, init_subscriber};

use semi_wallet::http_server::HttpServer;

#[tokio::main]
async fn main() {
    let cfg = config::Settings::new().expect("cannot parse configuration");
    let subscriber = get_subscriber("semi-wallet-server".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let http_server = HttpServer::build(cfg).await;
    http_server.run().await.unwrap();
}
