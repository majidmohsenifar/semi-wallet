use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum UserPlanError {
    #[snafu(display("cannot calculate expiry date for the plan"))]
    InvalidExpiration,
    #[snafu(display("{message}"))]
    Unexpected {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
