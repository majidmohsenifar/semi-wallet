use std::collections::HashMap;

use super::{
    bitpay::{self, BitpayProvider},
    error::PaymentError,
    stripe::{self, StripeProvider},
};

#[derive(Debug)]
pub struct CreatePaymentResult {}

#[derive(Debug)]
pub struct CreatePaymentParams {
    pub order_id: u64,
}

#[derive(Debug)]
pub struct CheckPaymentParams {}

#[derive(Debug)]
pub struct CheckPaymentResult {}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Provider {
    Stripe,
    Bitpay,
}

//#[derive(Clone)]
pub struct Service {
    //providers: Arc<HashMap<Provider, PaymentHandler>>,
    providers: HashMap<Provider, PaymentHandler>,
}

pub struct CreatePayment {
    provider: String,
}

enum PaymentHandler {
    Stripe(StripeProvider),
    Bitpay(BitpayProvider),
}

impl Service {
    pub fn new() -> Self {
        let stripe = stripe::StripeProvider::new();
        let bitpay = bitpay::BitpayProvider::new();
        let mut providers: HashMap<Provider, PaymentHandler> = HashMap::new();
        providers.insert(Provider::Stripe, PaymentHandler::Stripe(stripe));
        providers.insert(Provider::Bitpay, PaymentHandler::Bitpay(bitpay));
        Service { providers }
    }

    pub async fn create_payment(
        &self,
        params: CreatePaymentParams,
    ) -> Result<CreatePaymentResult, PaymentError> {
        println!("{:?}", params);
        let payment_provider = self.providers.get(&Provider::Stripe).unwrap();
        let p = match payment_provider {
            PaymentHandler::Stripe(stripe) => stripe.make_payment(params),
            PaymentHandler::Bitpay(bitpay) => bitpay.make_payment(params),
        };
        Ok(CreatePaymentResult {})
    }
}
