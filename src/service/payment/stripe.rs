use std::{collections::HashMap, str::FromStr};

use stripe::{
    CheckoutSession, CheckoutSessionId, Client, CreateCheckoutSession,
    CreateCheckoutSessionLineItems, CreateCheckoutSessionLineItemsPriceData,
};

use crate::{
    repository::models::PaymentStatus,
    service::payment::service::{MakePaymentResult, EXPIRE_DURATION},
};

use super::{
    error::PaymentError,
    service::{
        CheckPaymentParams, CheckPaymentResult, HandlerCheckPaymentResult, MakePaymentParams,
    },
};

const STRIPE_METADATA_PAYMENT_ID_KEY: &str = "payment_id";
const STRIPE_METADATA_ORDER_ID_KEY: &str = "order_id";
const STRIPE_METADATA_ENV_KEY: &str = "env";
const STRIPE_PRODUCT_MANE: &str = "Semi-wallet charge";

pub struct StripeProvider {
    client: Client,
}

impl StripeProvider {
    pub fn new(stripe_url: &str, stripe_secret: &str) -> Self {
        let client = stripe::Client::from_url(stripe_url, stripe_secret);
        StripeProvider { client }
    }

    pub async fn make_payment(
        &self,
        params: MakePaymentParams,
    ) -> Result<MakePaymentResult, PaymentError> {
        let client_reference_id = params.payment_id.to_string();

        let mut checkout_params = CreateCheckoutSession::new();
        checkout_params.currency = Some(stripe::Currency::USD);
        checkout_params.mode = Some(stripe::CheckoutSessionMode::Payment);
        checkout_params.client_reference_id = Some(&client_reference_id);
        checkout_params.expires_at = Some(EXPIRE_DURATION);
        //checkout_params.success_url = Some("");
        //checkout_params.cancel_url = Some("");
        checkout_params.metadata = Some(HashMap::from([
            (
                STRIPE_METADATA_PAYMENT_ID_KEY.to_string(),
                params.payment_id.to_string(),
            ),
            (
                STRIPE_METADATA_ORDER_ID_KEY.to_string(),
                params.order_id.to_string(),
            ),
            (STRIPE_METADATA_ENV_KEY.to_string(), "prod".to_string()), //TODO: must be read from
                                                                       //config
        ]));
        checkout_params.line_items = Some(vec![CreateCheckoutSessionLineItems {
            price_data: Some(CreateCheckoutSessionLineItemsPriceData {
                currency: stripe::Currency::USD,
                unit_amount: Some((params.amount * 100_f64) as i64),
                product_data: Some(stripe::CreateCheckoutSessionLineItemsPriceDataProductData {
                    name: STRIPE_PRODUCT_MANE.to_string(),
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
            return Err(PaymentError::Unexpected {
                message: "cannot create checkout session for stripe".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            });
        }

        let checkout_session_res = checkout_session_res.unwrap();
        let url = {
            if let Some(url) = checkout_session_res.url {
                url
            } else {
                return Err(PaymentError::StripeError {
                    message: "empty url".to_string(),
                });
            }
        };
        Ok(MakePaymentResult {
            url,
            external_id: checkout_session_res.id.to_string(),
        })
    }

    pub async fn check_payment(
        &self,
        params: CheckPaymentParams,
    ) -> Result<HandlerCheckPaymentResult, PaymentError> {
        let checkout_session_id = match CheckoutSessionId::from_str(&params.external_id) {
            Ok(id) => id,
            Err(e) => {
                return Err(PaymentError::Unexpected {
                    message: "cannot get checkout session_id".to_string(),
                    source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
                })
            }
        };
        let c_session = match stripe::CheckoutSession::retrieve(
            &self.client,
            &checkout_session_id,
            &[],
        )
        .await
        {
            Ok(s) => s,
            Err(e) => {
                return Err(PaymentError::Unexpected {
                    message: "cannot retrieve checkout session".to_string(),
                    source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
                })
            }
        };
        let status = match c_session.status {
            Some(s) => match s {
                stripe::CheckoutSessionStatus::Open => PaymentStatus::Created,
                stripe::CheckoutSessionStatus::Expired => PaymentStatus::Failed,
                stripe::CheckoutSessionStatus::Complete => PaymentStatus::Completed,
            },
            None => {
                return Err(PaymentError::StripeError {
                    message: "cannot retrieve checkout session".to_string(),
                });
            }
        };

        let metadata = serde_json::to_string(&c_session).map_err(|e| PaymentError::Unexpected {
            message: "cannot retrieve checkout session".to_string(),
            source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
        })?;
        let amount = c_session.amount_total.ok_or(PaymentError::StripeError {
            message: "amount_total is empty".to_string(),
        })?;
        let amount = amount as f64 / 100.0;
        Ok(HandlerCheckPaymentResult {
            amount,
            status,
            metadata,
        })
    }
}
