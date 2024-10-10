use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum UserCoinError {
    #[snafu(display("coin not found"))]
    CoinNotFound,
    #[snafu(display("user coin not found"))]
    UserCoinNotFound,
    #[snafu(display("{message}"))]
    Unexpected {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
