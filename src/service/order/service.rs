use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use tracing::error;
use validator::{Validate, ValidationError};

use crate::repository::db::Repository;
use crate::repository::models::{OrderStatus, User};
use crate::repository::order::CreateOrderArgs;
use crate::service::payment::service::{CreatePaymentParams, Provider, Service as PaymentService};
use crate::service::plan::service::{
    Service as PlanService, PLAN_CODE_12_MONTH, PLAN_CODE_1_MONTH, PLAN_CODE_3_MONTH,
    PLAN_CODE_6_MONTH,
};

use super::error::OrderError;

pub struct Service {
    db: Pool<Postgres>,
    repo: Repository,
    plan_service: PlanService,
    payment_service: PaymentService,
}

#[derive(serde::Deserialize, Validate)]
pub struct CreateOrderParams {
    #[validate(custom(function = "validate_plan_code"))]
    pub plan_code: String,
    #[validate(custom(function = "validate_payment_provider"))]
    pub payment_provider: String,
}

fn validate_plan_code(plan_code: &str) -> Result<(), ValidationError> {
    if [
        PLAN_CODE_1_MONTH,
        PLAN_CODE_3_MONTH,
        PLAN_CODE_6_MONTH,
        PLAN_CODE_12_MONTH,
    ]
    .contains(&plan_code)
    {
        return Ok(());
    }
    Err(ValidationError::new("invalid plan_code"))
}

fn validate_payment_provider(payment_provider: &str) -> Result<(), ValidationError> {
    if Provider::from(payment_provider).is_some() {
        return Ok(());
    }
    Err(ValidationError::new("invalid payment_provider"))
}

#[derive(Serialize, Deserialize)]
pub struct CreateOrderResult {
    pub id: i64,
    pub status: String,
    pub payment_url: String,
}

#[derive(serde::Deserialize)]
pub struct OrderDetailParams {
    pub id: i64,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct OrderDetailResult {
    pub id: i64,
}

impl Service {
    pub fn new(
        db: Pool<Postgres>,
        repo: Repository,
        plan_service: PlanService,
        payment_service: PaymentService,
    ) -> Self {
        Service {
            db,
            repo,
            plan_service,
            payment_service,
        }
    }

    pub async fn create_order(
        &self,
        user: User,
        params: CreateOrderParams,
    ) -> Result<CreateOrderResult, OrderError> {
        let payment_provider = match Provider::from(&params.payment_provider) {
            Some(pp) => pp,
            None => return Err(OrderError::InvalidPaymentProvider),
        };
        let plan = self.plan_service.get_plan_by_code(&params.plan_code).await;
        if let Err(e) = plan {
            match e {
                sqlx::Error::RowNotFound => {
                    return Err(OrderError::PlanNotFound {
                        code: params.plan_code,
                    });
                }
                _ => {
                    return Err(OrderError::Unexpected {
                        message: "cannot get plan".to_string(),
                        source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
                    });
                }
            }
        }
        let plan = plan.unwrap();

        let db_tx = self.db.begin().await;
        if let Err(e) = db_tx {
            error!("cannot start db_tx due to err: {e}");
            return Err(OrderError::Unexpected {
                message: "cannot start transaction".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            });
        }

        let mut db_tx = db_tx.unwrap();
        let order = self
            .repo
            .create_order(
                &mut db_tx,
                CreateOrderArgs {
                    user_id: user.id,
                    plan_id: plan.id,
                    total: plan.price,
                    status: OrderStatus::Created,
                },
            )
            .await;
        if let Err(e) = order {
            let _ = db_tx.rollback().await;
            return Err(OrderError::Unexpected {
                message: "cannot create order".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            });
        }
        let order = order.unwrap();

        let payment = self
            .payment_service
            .create_payment(
                &mut db_tx,
                CreatePaymentParams {
                    order_id: order.id,
                    user_id: order.user_id,
                    amount: order.total,
                    payment_provider,
                },
            )
            .await;

        if let Err(e) = payment {
            let _ = db_tx.rollback().await;
            println!("errorrrrrrrrrrrrrrrrrrrrrr {e:?}");
            return Err(OrderError::Unexpected {
                message: "cannot create payment".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            });
        }

        let payment = payment.unwrap();
        let tx_res = db_tx.commit().await;
        if let Err(e) = tx_res {
            //TODO: shouldn't we rollback? but how
            //let _ = db_tx.rollback().await;
            return Err(OrderError::Unexpected {
                message: "cannot insert to db".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            });
        }
        Ok(CreateOrderResult {
            id: order.id,
            status: "CREATED".to_string(), //TODO: handle this later
            payment_url: payment.url,
        })
    }

    pub async fn order_detail(
        &self,
        user: User,
        params: OrderDetailParams,
    ) -> Result<OrderDetailResult, OrderError> {
        let conn = self.db.acquire().await;
        if let Err(e) = conn {
            error!("cannot acquire db conn due to err {e}");
            return Err(OrderError::Unexpected {
                message: "cannot get order from db".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            });
        }
        let mut conn = conn.unwrap();
        let order = self.repo.get_order_by_id(&mut conn, params.id).await;
        if let Err(e) = order {
            match e {
                sqlx::Error::RowNotFound => return Err(OrderError::NotFound { id: params.id }),
                _ => {
                    error!("cannot get order due to err {e}");
                    return Err(OrderError::Unexpected {
                        message: "cannot get order from db".to_string(),
                        source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
                    });
                }
            };
        }
        let order = order.unwrap();
        if order.user_id != user.id {
            return Err(OrderError::NotFound { id: params.id });
        }
        Ok(OrderDetailResult { id: order.id })
    }
}
