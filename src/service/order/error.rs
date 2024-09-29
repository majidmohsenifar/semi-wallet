use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum OrderError {
    #[snafu(display("order with id {id} not found"))]
    NotFound { id: i64 },
    #[snafu(display("plan with code {code} not found"))]
    PlanNotFound { code: String },
    #[snafu(display("invalid payment provider"))]
    InvalidPaymentProvider,
    #[snafu(display("{message}"))]
    Unexpected {
        message: String,
        source: Box<dyn std::error::Error>,
    },
}
