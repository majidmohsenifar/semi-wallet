use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum UserCoinError {
    #[snafu(display("coin or network not found"))]
    CoinOrNetworkNotFound,
    #[snafu(display("user coin not found"))]
    UserCoinNotFound,
    #[snafu(display("user does not have any plan"))]
    UserPlanNotFound,
    #[snafu(display("user plan is expired"))]
    UserPlanExpired,
    #[snafu(display("{message}"))]
    Unexpected {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
