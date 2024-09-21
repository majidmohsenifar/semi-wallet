use super::service::{
    CheckPaymentParams, CheckPaymentResult, CreatePaymentParams, CreatePaymentResult,
};

pub struct BitpayProvider {}

impl BitpayProvider {
    pub fn new() -> Self {
        BitpayProvider {}
    }
    pub fn make_payment(&self, _params: CreatePaymentParams) -> CreatePaymentResult {
        todo!("impl later")
    }
    pub fn check_payment(&self, _params: CheckPaymentParams) -> CheckPaymentResult {
        todo!("impl later")
    }
}
