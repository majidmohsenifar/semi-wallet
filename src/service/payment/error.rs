use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum PaymentError {
    #[snafu(display("something went wrong with amount calculation"))]
    InvalidAmount,
    #[snafu(display("payment url is empty"))]
    EmptyUrl,
    #[snafu(display("{message}"))]
    Unexpected {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
