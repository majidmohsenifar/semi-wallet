use super::{
    error::PaymentError,
    service::{
        CheckPaymentHandlerResult, CheckPaymentParams, MakePaymentParams, MakePaymentResult,
    },
};

pub struct BitpayProvider {}

impl BitpayProvider {
    pub fn new() -> Self {
        BitpayProvider {}
    }
    pub async fn make_payment(
        &self,
        _params: MakePaymentParams,
    ) -> Result<MakePaymentResult, PaymentError> {
        todo!("impl later")
        //Ok(MakePaymentResult {
        //url: "".to_string(),         //TODO: handle this later
        //external_id: "".to_string(), //TODO: handle this later
        //})
    }
    pub async fn check_payment(
        &self,
        _params: CheckPaymentParams,
    ) -> Result<CheckPaymentHandlerResult, PaymentError> {
        todo!("impl later")
    }
}
