use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum BlockchainError {
    #[snafu(display("invalid blockchain"))]
    InvalidBlockchain,
    #[snafu(display("invalid addr"))]
    InvalidAddress,
    #[snafu(display("{blockchain} blockchain does not support token"))]
    TokenNotSupported { blockchain: String },
    #[snafu(display("status code is {code}"))]
    UnsuccessfulStatusCode { code: u16 },
    #[snafu(display("calling tron contract was not successful"))]
    UnsuccessfulTronContractCall,
    #[snafu(display("{message}"))]
    Unexpected {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
