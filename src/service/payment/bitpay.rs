use super::{
    error::PaymentError,
    service::{
        CheckPaymentParams, HandlerCheckPaymentResult, MakePaymentParams, MakePaymentResult,
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
    ) -> Result<HandlerCheckPaymentResult, PaymentError> {
        todo!("impl later")
    }
}
