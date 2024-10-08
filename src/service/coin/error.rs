use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum CoinError {
    #[snafu(display("coin not found"))]
    NotFound,
    #[snafu(display("{message}"))]
    Unexpected {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
