use core::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum OrderError {
    NotFound,
    Unknown,
    PlanNotFound,
    InvalidPaymentProvider,
}

impl Error for OrderError {}

impl fmt::Display for OrderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error")
    }
}
