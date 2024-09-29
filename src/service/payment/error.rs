use core::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum PaymentError {
    Unexpected,
}

impl Error for PaymentError {}

impl fmt::Display for PaymentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error")
    }
}
