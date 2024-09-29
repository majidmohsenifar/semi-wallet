use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum CoinError {
    #[snafu(display("coin with symbol {symbol} not found"))]
    NotFound { symbol: String },
    #[snafu(display("{message}"))]
    Unexpected {
        message: String,
        source: Box<dyn std::error::Error>,
    },
}
