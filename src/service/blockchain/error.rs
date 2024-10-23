use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum BlockchainError {
    #[snafu(display("invalid blockchain"))]
    InvalidBlockchain,
    #[snafu(display("invalid addr"))]
    InvalidAddress,
    #[snafu(display("{message}"))]
    Unexpected {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
