use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum UserError {
    #[snafu(display("{message}"))]
    Unexpected {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
