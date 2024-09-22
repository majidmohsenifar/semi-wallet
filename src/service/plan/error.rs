use core::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum PlanError {
    NotFound,
    Unknown,
}

impl Error for PlanError {}

impl fmt::Display for PlanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error")
    }
}
