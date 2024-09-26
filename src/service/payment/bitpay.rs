use super::service::{
    CheckPaymentParams, CheckPaymentResult, MakePaymentParams, MakePaymentResult,
};

pub struct BitpayProvider {}

impl BitpayProvider {
    pub fn new() -> Self {
        BitpayProvider {}
    }
    pub async fn make_payment(&self, _params: MakePaymentParams) -> Result<MakePaymentResult, ()> {
        Ok(MakePaymentResult {
            url: "".to_string(),         //TODO: handle this later
            external_id: "".to_string(), //TODO: handle this later
        })
    }
    pub fn check_payment(&self, _params: CheckPaymentParams) -> CheckPaymentResult {
        todo!("impl later")
    }
}
