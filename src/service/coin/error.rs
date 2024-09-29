use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum CoinError {
    #[snafu(display("coin with symbol {symbol} not found"))]
    NotFound { symbol: String },
    #[snafu(display("unexpected error"))]
    Unexpected,
}
