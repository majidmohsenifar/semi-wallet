use config::Config;

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Settings {
    pub db: DbConfig,
    pub redis: RedisConfig,
    pub server: ServerConfig,
    pub stripe: StripeConfig,
    pub jwt: JwtConfig,
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

//TODO: why not associative func?
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let cfg = Config::builder()
        //.add_source(config::File::with_name(".env"))
        .add_source(config::Environment::default().separator("_"))
        .build()?;
    cfg.try_deserialize::<Settings>()
}
