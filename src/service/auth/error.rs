use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum AuthError {
    #[snafu(display("email already exist"))]
    EmailAlreadyTaken,
    #[snafu(display("invalid credentials"))]
    InvalidCredentials,
    #[snafu(display("invalid token"))]
    InvalidToken,
    #[snafu(display("{message}"))]
    Unexpected {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
