use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum PlanError {
    #[snafu(display("plan with code {code} not found"))]
    NotFound { code: String },
    #[snafu(display("something went wrong with price conversion"))]
    InvalidPrice,
    #[snafu(display("{message}"))]
    Unexpected {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
