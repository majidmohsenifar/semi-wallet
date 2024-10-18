use config::Config;
use dotenv;

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Settings {
    pub db: DbConfig,
    pub redis: RedisConfig,
    pub server: ServerConfig,
    pub stripe: StripeConfig,
    pub jwt: JwtConfig,
    pub btc: BlockchainConfig,
    pub eth: BlockchainConfig,
    pub sol: BlockchainConfig,
    pub trx: BlockchainConfig,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct DbConfig {
    pub dsn: String,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct RedisConfig {
    pub uri: String,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct ServerConfig {
    pub address: String,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct StripeConfig {
    pub url: String,
    pub secret: String,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct JwtConfig {
    pub secret: String,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct BlockchainConfig {
    pub url: String,
    pub decimals: u8,
    pub blockbook_support: bool,
}

impl Settings {
    pub fn new() -> Result<Self, config::ConfigError> {
        #[cfg(debug_assertions)]
        dotenv::dotenv().ok();

        let cfg = Config::builder()
            //.add_source(config::File::with_name(".env"))
            .add_source(config::Environment::default().separator("__"))
            .build()?;
        cfg.try_deserialize::<Settings>()
    }
}
