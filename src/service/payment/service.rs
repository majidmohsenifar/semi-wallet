use bigdecimal::ToPrimitive;
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;
use std::{collections::HashMap, fmt::Display};
use utoipa::ToSchema;

use sqlx::{Pool, Postgres};
use std::sync::Arc;

use crate::repository::{
    db::Repository,
    models::{Payment, PaymentStatus},
    payment::CreatePaymentArgs,
};

pub const PAYMENT_PROVIDER_STRIPE: &str = "STRIPE";
pub const PAYMENT_PROVIDER_BITPAY: &str = "BITPAY";
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
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub struct CheckPaymentHandlerResult {
    pub status: PaymentStatus,
    pub amount: f64,
    pub metadata: String,
}

#[derive(Debug)]
pub struct CreatePaymentParams {
    pub order_id: i64,
    pub user_id: i64,
    pub amount: BigDecimal,
    pub payment_provider: Provider,
}

#[derive(Debug)]
pub struct CheckPaymentParams {
    pub external_id: String,
    pub amount: f64,
}

#[derive(Debug)]
pub struct CheckPaymentResult {
    pub status: PaymentStatus,
    pub amount: f64,
    pub metadata: String,
    pub payment: Payment,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Provider {
    Stripe,
    Bitpay,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaymentProvider {
    pub code: String,
    pub enabled: bool,
}

impl Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::Stripe => {
                write!(f, "{}", PAYMENT_PROVIDER_STRIPE)
            }
            Provider::Bitpay => {
                write!(f, "{}", PAYMENT_PROVIDER_BITPAY)
            }
        }
    }
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

#[derive(Clone)]
pub struct Service {
    db: Pool<Postgres>,
    repo: Repository,
    providers: Arc<HashMap<Provider, PaymentHandler>>,
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
            providers: Arc::new(providers),
        }
    }

    pub async fn create_payment(
        &self,
        db_tx: &mut sqlx::Transaction<'_, Postgres>,
        params: CreatePaymentParams,
    ) -> Result<CreatePaymentResult, PaymentError> {
        let payment = self
            .repo
            .create_payment(
                db_tx,
                CreatePaymentArgs {
                    user_id: params.user_id,
                    order_id: params.order_id,
                    amount: params.amount,
                    payment_provider_code: params.payment_provider.to_string(),
                    status: crate::repository::models::PaymentStatus::Created,
                },
            )
            .await;

        if let Err(e) = payment {
            tracing::error!("cannot create_payment due to err: {}", e);
            return Err(PaymentError::Unexpected {
                message: "cannot create payment".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            });
        }
        let payment = payment.unwrap();
        let amount = match payment.amount.to_f64() {
            Some(float) => float,
            None => return Err(PaymentError::InvalidAmount),
        };

        let payment_handler = self.providers.get(&params.payment_provider).unwrap();
        let make_payment_params = MakePaymentParams {
            payment_id: payment.id,
            amount,
            order_id: params.order_id,
            extra_data: HashMap::new(),
        };
        let make_payment_result = match payment_handler {
            PaymentHandler::Stripe(stripe) => stripe.make_payment(make_payment_params).await,
            PaymentHandler::Bitpay(bitpay) => bitpay.make_payment(make_payment_params).await,
        };

        if let Err(e) = make_payment_result {
            tracing::error!("cannot make_payment due to err: {}", e);
            return Err(PaymentError::Unexpected {
                message: "cannot make payment".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            });
        }

        let make_payment_result = make_payment_result.unwrap();
        let update_payment_result = self
            .repo
            .update_payment_external_id_payment_url_expires_at(
                db_tx,
                payment.id,
                &make_payment_result.external_id,
                &make_payment_result.url,
                make_payment_result.expires_at,
            )
            .await;
        if let Err(e) = update_payment_result {
            tracing::error!("cannot update_payment_result due to err: {}", e);
            return Err(PaymentError::Unexpected {
                message: "cannot make payment".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            });
        }

        Ok(CreatePaymentResult {
            payment,
            url: make_payment_result.url,
        })
    }

    pub async fn check_payment(&self, id: i64) -> Result<CheckPaymentResult, PaymentError> {
        //TODO: handle all unwrap in this method
        let p = self.repo.get_payment_by_id(&self.db, id).await;
        let p = match p {
            Ok(payment) => payment,
            Err(e) => match e {
                sqlx::Error::RowNotFound => return Err(PaymentError::NotFound { id }),
                other => {
                    tracing::error!("cannot get_payment_by_id due to err: {}", e);
                    return Err(PaymentError::Unexpected {
                        message: "cannot get payment by id from db".to_string(),
                        source: Box::new(other) as Box<dyn std::error::Error + Send + Sync>,
                    });
                }
            },
        };
        let payment_provider = Provider::from(&p.payment_provider_code).unwrap();
        let payment_handler = self.providers.get(&payment_provider).unwrap();
        let check_payment_params = CheckPaymentParams {
            amount: p.amount.to_f64().unwrap(),
            external_id: p.external_id.clone().unwrap(),
        };
        let handler_check_payment_result = match payment_handler {
            PaymentHandler::Stripe(stripe) => stripe.check_payment(check_payment_params).await?,
            PaymentHandler::Bitpay(bitpay) => bitpay.check_payment(check_payment_params).await?,
        };
        //check_payment_result.status;

        Ok(CheckPaymentResult {
            status: handler_check_payment_result.status,
            amount: handler_check_payment_result.amount,
            metadata: handler_check_payment_result.metadata,
            payment: p,
        })
    }

    pub async fn update_payment_status_metadata(
        &self,
        db_tx: &mut sqlx::Transaction<'_, Postgres>,
        payment_id: i64,
        status: PaymentStatus,
        metadata: Option<sqlx::types::JsonValue>,
    ) -> Result<(), sqlx::Error> {
        self.repo
            .update_payment_status_metadata(db_tx, payment_id, status, metadata)
            .await?;
        Ok(())
    }

    pub async fn get_last_payment_by_order_id(
        &self,
        order_id: i64,
    ) -> Result<Payment, sqlx::Error> {
        self.repo
            .get_last_payment_by_order_id(&self.db, order_id)
            .await
    }

    pub async fn get_payment_providers(&self) -> Vec<PaymentProvider> {
        vec![
            PaymentProvider {
                code: PAYMENT_PROVIDER_STRIPE.to_string(),
                enabled: true,
            },
            PaymentProvider {
                code: PAYMENT_PROVIDER_BITPAY.to_string(),
                enabled: true,
            },
        ]
    }
}
