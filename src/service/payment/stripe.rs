use std::collections::HashMap;

use stripe::{
    CheckoutSession, Client, CreateCheckoutSession, CreateCheckoutSessionLineItems,
    CreateCheckoutSessionLineItemsPriceData,
};

use crate::service::payment::service::{MakePaymentResult, EXPIRE_DURATION};

use super::service::{CheckPaymentParams, CheckPaymentResult, MakePaymentParams};

pub struct StripeProvider {
    client: Client,
}

impl StripeProvider {
    pub fn new(stripe_url: &str, stripe_secret: &str) -> Self {
        let client = stripe::Client::from_url(stripe_url, stripe_secret);
        StripeProvider { client }
    }

    pub async fn make_payment(&self, params: MakePaymentParams) -> Result<MakePaymentResult, ()> {
        let client_reference_id = params.payment_id.to_string();

        let mut checkout_params = CreateCheckoutSession::new();
        checkout_params.currency = Some(stripe::Currency::USD);
        checkout_params.mode = Some(stripe::CheckoutSessionMode::Payment);
        checkout_params.client_reference_id = Some(&client_reference_id);
        checkout_params.expires_at = Some(EXPIRE_DURATION);
        checkout_params.success_url = Some(""); //TODO: handle this later, must be read from config
        checkout_params.cancel_url = Some(""); //TODO: handle this later
        checkout_params.metadata = Some(HashMap::from([
            ("payment_id".to_string(), params.payment_id.to_string()), //TODO: better to make key as consntant
            ("order_id".to_string(), params.order_id.to_string()), //TODO: better to make key as constant
            ("env".to_string(), "prod".to_string()),               //TODO: handle this later
        ]));
        checkout_params.line_items = Some(vec![CreateCheckoutSessionLineItems {
            price_data: Some(CreateCheckoutSessionLineItemsPriceData {
                currency: stripe::Currency::USD,
                unit_amount: Some((params.amount * 100 as f64) as i64),
                product_data: Some(stripe::CreateCheckoutSessionLineItemsPriceDataProductData {
                    name: "semi-wallet".to_string(), //TODO: change this later, make it constant
                    ..Default::default()
                }),
                ..Default::default()
            }),
            quantity: Some(1),
            ..Default::default()
        }]);
        //replace unwarp with something proper
        let checkout_session_res = CheckoutSession::create(&self.client, checkout_params).await;
        if let Err(e) = checkout_session_res {
            return Err(()); //TODO: handle this later
        }

        let checkout_session_res = checkout_session_res.unwrap();
        let url = {
            if let Some(url) = checkout_session_res.url {
                url
            } else {
                return Err(());
            }
        };
        Ok(MakePaymentResult {
            url,
            external_id: checkout_session_res.id.to_string(),
        })
    }

    pub fn check_payment(&self, _params: CheckPaymentParams) -> CheckPaymentResult {
        todo!("impl later")
    }
}
