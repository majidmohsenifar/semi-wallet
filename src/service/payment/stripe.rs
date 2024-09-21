use super::service::{
    CheckPaymentParams, CheckPaymentResult, CreatePaymentParams, CreatePaymentResult,
};

pub struct StripeProvider {}

impl StripeProvider {
    pub fn new() -> Self {
        StripeProvider {}
    }
    pub fn make_payment(&self, params: CreatePaymentParams) -> CreatePaymentResult {
        println!("from stripe {:#?}", params);
        CreatePaymentResult {}
    }
    pub fn check_payment(&self, _params: CheckPaymentParams) -> CheckPaymentResult {
        todo!("impl later")
    }
}
