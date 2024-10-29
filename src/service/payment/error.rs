use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum PaymentError {
    #[snafu(display("something went wrong with amount calculation"))]
    InvalidAmount,
    #[snafu(display("stripe error: {message}"))]
    StripeError { message: String },
    #[snafu(display("payment with id {id} not found"))]
    NotFound { id: i64 },
    #[snafu(display("invalid payment provider"))]
    InvalidPaymentProvider,
    #[snafu(display("{message}"))]
    Unexpected {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
