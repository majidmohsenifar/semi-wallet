use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum BlockchainError {
    #[snafu(display("invalid blockchain"))]
    InvalidBlockchain,
}
