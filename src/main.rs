use semi_wallet::config;

use semi_wallet::http_server::HttpServer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    let cfg = config::get_configuration().expect("cannot parse configuration");
    //TODO: improve the config for tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let http_server = HttpServer::build(cfg).await;
    http_server.run().await.unwrap();
}
