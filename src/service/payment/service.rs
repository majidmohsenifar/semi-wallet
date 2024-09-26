use std::collections::HashMap;

use sqlx::{Pool, Postgres};

use crate::repository::{db::Repository, models::Payment};

const PAYMENT_PROVIDER_STRIPE: &str = "STRIPE";
const PAYMENT_PROVIDER_BITPAY: &str = "BITPAY";
pub const EXPIRE_DURATION: i64 = 30; //30 min

use super::{
    bitpay::{self, BitpayProvider},
    error::PaymentError,
    stripe::{self, StripeProvider},
};

pub struct CreatePaymentResult {
    pub payment: Payment,
    pub url: String,
}

pub struct MakePaymentParams {
    pub amount: f64,
    pub payment_id: i64,
    pub order_id: i64,
    pub extra_data: HashMap<String, String>,
}

pub struct MakePaymentResult {
    pub url: String,
    pub external_id: String,
}

#[derive(Debug)]
pub struct CreatePaymentParams {
    pub order_id: i64,
    pub user_id: i64,
    pub amount: f64,
    pub payment_provider: Provider,
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

impl Provider {
    pub fn from(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            PAYMENT_PROVIDER_STRIPE => Some(Provider::Stripe),
            PAYMENT_PROVIDER_BITPAY => Some(Provider::Bitpay),
            _ => None,
        }
    }
}

//#[derive(Clone)]
pub struct Service {
    db: Pool<Postgres>,
    repo: Repository,
    providers: HashMap<Provider, PaymentHandler>,
    //providers: Arc<HashMap<Provider, PaymentHandler>>,
}

enum PaymentHandler {
    Stripe(StripeProvider),
    Bitpay(BitpayProvider),
}

impl Service {
    pub fn new(
        db: Pool<Postgres>,
        repo: Repository,
        stripe_url: &str,
        stripe_secret: &str,
    ) -> Self {
        let stripe = stripe::StripeProvider::new(stripe_url, stripe_secret);
        let bitpay = bitpay::BitpayProvider::new();
        let mut providers: HashMap<Provider, PaymentHandler> = HashMap::new();
        providers.insert(Provider::Stripe, PaymentHandler::Stripe(stripe));
        providers.insert(Provider::Bitpay, PaymentHandler::Bitpay(bitpay));
        Service {
            db,
            repo,
            providers,
        }
    }

    pub async fn create_payment(
        &self,
        db_tx: &mut sqlx::Transaction<'_, Postgres>,
        params: CreatePaymentParams,
    ) -> Result<CreatePaymentResult, PaymentError> {
        //TODO: insert into db
        let payment = self
            .repo
            .create_payment(
                db_tx,
                crate::repository::payment::CreatePaymentArgs {
                    user_id: params.user_id,
                    order_id: params.order_id,
                    amount: params.amount,
                    status: crate::repository::models::PaymentStatus::Created,
                },
            )
            .await;

        if let Err(e) = payment {
            //TODO: we should log here
            println!("{e}");
            return Err(PaymentError::Unknown);
        }

        let payment = payment.unwrap();

        let payment_provider = self.providers.get(&params.payment_provider).unwrap();
        let make_payment_params = MakePaymentParams {
            payment_id: payment.id,
            amount: params.amount,
            order_id: params.order_id,
            extra_data: HashMap::new(),
        };
        let make_payment_result = match payment_provider {
            PaymentHandler::Stripe(stripe) => stripe.make_payment(make_payment_params).await,
            PaymentHandler::Bitpay(bitpay) => bitpay.make_payment(make_payment_params).await,
        };

        if let Err(e) = make_payment_result {
            //TODO: we should log here and handle error better
            println!("{:#?}", e);
            return Err(PaymentError::Unknown);
        }

        let make_payment_result = make_payment_result.unwrap();
        let update_payment_result = self
            .repo
            .update_payment_external_id(db_tx, payment.id, make_payment_result.external_id)
            .await;
        if let Err(e) = update_payment_result {
            //TODO: we should log here and handle error better
            println!("{:#?}", e);
            return Err(PaymentError::Unknown);
        }

        Ok(CreatePaymentResult {
            payment,
            url: make_payment_result.url,
        })
    }
}
