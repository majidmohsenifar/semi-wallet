use super::service::{
    CheckPaymentParams, CheckPaymentResult, CreatePaymentParams, MakePaymentResult,
};

pub struct BitpayProvider {}

impl BitpayProvider {
    pub fn new() -> Self {
        BitpayProvider {}
    }
    pub fn make_payment(&self, _params: CreatePaymentParams) -> Result<MakePaymentResult, ()> {
        Ok(MakePaymentResult {
            url: "".to_string(),         //TODO: handle this later
            external_id: "".to_string(), //TODO: handle this later
        })
    }
    pub fn check_payment(&self, _params: CheckPaymentParams) -> CheckPaymentResult {
        todo!("impl later")
    }
}
