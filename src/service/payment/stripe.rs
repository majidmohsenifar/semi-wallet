use crate::service::payment::service::MakePaymentResult;

use super::service::{CheckPaymentParams, CheckPaymentResult, CreatePaymentParams};

pub struct StripeProvider {}

impl StripeProvider {
    pub fn new() -> Self {
        StripeProvider {}
    }
    pub fn make_payment(&self, params: CreatePaymentParams) -> Result<MakePaymentResult, ()> {
        //TODO: handle this later
        println!("{:#?}", params);
        Ok(MakePaymentResult {
            url: "".to_string(),         //TODO: handle this later
            external_id: "".to_string(), //TODO: handle this later
        })
    }
    pub fn check_payment(&self, _params: CheckPaymentParams) -> CheckPaymentResult {
        todo!("impl later")
    }
}
