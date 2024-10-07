use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use stripe::{EventObject, EventType};
use tracing::error;
use validator::{Validate, ValidationError};

use crate::repository::db::Repository;
use crate::repository::models::{Order, OrderStatus, Payment, PaymentStatus, User};
use crate::repository::order::CreateOrderArgs;
use crate::service::payment::service::{CreatePaymentParams, Provider, Service as PaymentService};

use crate::service::plan::service::{
    Service as PlanService, PLAN_CODE_12_MONTH, PLAN_CODE_1_MONTH, PLAN_CODE_3_MONTH,
    PLAN_CODE_6_MONTH,
};
use crate::service::user_plan::service::Service as UserPlanService;

use super::error::OrderError;

pub struct Service {
    db: Pool<Postgres>,
    repo: Repository,
    plan_service: PlanService,
    payment_service: PaymentService,
    user_plan_service: UserPlanService,
    stripe_secret: String,
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
        user_plan_service: UserPlanService,
        stripe_secret: String,
    ) -> Self {
        Service {
            db,
            repo,
            plan_service,
            payment_service,
            user_plan_service,
            stripe_secret,
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
            return Err(OrderError::Unexpected {
                message: "cannot create payment".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            });
        }

        let payment = payment.unwrap();
        let commit_res = db_tx.commit().await;
        if let Err(e) = commit_res {
            //TODO: shouldn't we rollback? but how, the commit causes move of db_tx
            //let _ = db_tx.rollback().await;
            return Err(OrderError::Unexpected {
                message: "cannot commit changes to db".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            });
        }
        let status =
            serde_json::to_string(&OrderStatus::Created).map_err(|e| OrderError::Unexpected {
                message: "cannot convert status".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            })?;
        Ok(CreateOrderResult {
            id: order.id,
            status,
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

    pub async fn handle_stripe_webhook(
        &self,
        stripe_signature_header: &str,
        request_body: &str,
    ) -> Result<(), OrderError> {
        let event = stripe::Webhook::construct_event(
            request_body,
            stripe_signature_header,
            &self.stripe_secret,
        );
        let event = match event {
            Err(e) => {
                return Err(OrderError::Unexpected {
                    message: "cannot construct stripe webhook event".to_string(),
                    source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
                });
            }
            Ok(res) => res,
        };
        //we only cares about checkout session completed and expired
        //so if the type is not one of those we return
        if event.type_ != EventType::CheckoutSessionCompleted
            && event.type_ != EventType::CheckoutSessionExpired
        {
            return Ok(());
        }
        let mut payment_id: i64 = 0;
        if let EventObject::CheckoutSession(session) = event.data.object {
            payment_id = match session.client_reference_id {
                Some(r_id) => match r_id.parse::<i64>() {
                    Ok(id) => id,
                    Err(_) => {
                        return Err(OrderError::InvalidStripeReferenceID { id: r_id });
                    }
                },
                None => {
                    ////TODO: this error is not correct, we should return better error related to object
                    return Err(OrderError::InvalidStripeReferenceID { id: "".to_string() });
                }
            }
        };
        if payment_id == 0 {
            //TODO: this error is not correct, we should return better error related to invalid
            //data
            return Err(OrderError::InvalidStripeReferenceID { id: "".to_string() });
        }

        self.check_payment_and_finalize_order(payment_id).await
    }

    pub async fn check_payment_and_finalize_order(
        &self,
        payment_id: i64,
    ) -> Result<(), OrderError> {
        let check_payment = self
            .payment_service
            .check_payment(payment_id)
            .await
            .unwrap();

        if check_payment.status == PaymentStatus::Created {
            return Ok(());
        }

        let conn = self.db.acquire().await;
        if let Err(e) = conn {
            error!("cannot acquire db conn due to err {e}");
            return Err(OrderError::Unexpected {
                message: "cannot get order from db".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            });
        }
        let mut conn = conn.unwrap();

        let o = self
            .repo
            .get_order_by_id(&mut conn, check_payment.payment.order_id)
            .await
            .unwrap();
        match check_payment.status {
            PaymentStatus::Created => Ok(()),
            PaymentStatus::Failed => {
                self.handle_successful_payment(check_payment.payment, o, &check_payment.metadata)
                    .await
            }
            PaymentStatus::Completed => {
                self.handle_failed_payment(check_payment.payment, o, &check_payment.metadata)
                    .await
            }
        }
    }

    async fn handle_successful_payment(
        &self,
        p: Payment,
        o: Order,
        metadata: &str,
    ) -> Result<(), OrderError> {
        let plan = self
            .plan_service
            .get_plan_by_id(o.plan_id)
            .await
            .map_err(|e| OrderError::Unexpected {
                message: "cannot get plan".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            })?;

        let mut db_tx = self.db.begin().await.map_err(|e| OrderError::Unexpected {
            message: "cannot start db transaction".to_string(),
            source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
        })?;

        let update_payment_res = self
            .payment_service
            .update_payment_status_metadata(&mut db_tx, p.id, PaymentStatus::Completed, metadata)
            .await;

        if let Err(e) = update_payment_res {
            let _ = db_tx.rollback().await;
            return Err(OrderError::Unexpected {
                message: "cannot update payment status".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            });
        }

        let update_order_res = self
            .repo
            .update_order_status(&mut db_tx, o.id, OrderStatus::Completed)
            .await;

        if let Err(e) = update_order_res {
            let _ = db_tx.rollback().await;
            return Err(OrderError::Unexpected {
                message: "cannot update order status".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            });
        }

        let create_user_plan_res = self
            .user_plan_service
            .create_user_plan_or_update_expires_at(&mut db_tx, o.user_id, plan)
            .await;

        if let Err(e) = create_user_plan_res {
            let _ = db_tx.rollback().await;
            return Err(OrderError::Unexpected {
                message: "cannot create or update user_plan".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            });
        }

        let commit_res = db_tx.commit().await;
        if let Err(e) = commit_res {
            //TODO: shouldn't we rollback? but how, the commit causes move of db_tx
            //let _ = db_tx.rollback().await;
            return Err(OrderError::Unexpected {
                message: "cannot commit changes to db".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            });
        }
        Ok(())
    }

    async fn handle_failed_payment(
        &self,
        p: Payment,
        o: Order,
        metadata: &str,
    ) -> Result<(), OrderError> {
        let mut db_tx = self.db.begin().await.map_err(|e| OrderError::Unexpected {
            message: "cannot start db transaction".to_string(),
            source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
        })?;

        let update_payment_res = self
            .payment_service
            .update_payment_status_metadata(&mut db_tx, p.id, PaymentStatus::Failed, metadata)
            .await;

        if let Err(e) = update_payment_res {
            let _ = db_tx.rollback().await;
            return Err(OrderError::Unexpected {
                message: "cannot update payment status".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            });
        }

        let update_order_res = self
            .repo
            .update_order_status(&mut db_tx, o.id, OrderStatus::Failed)
            .await;

        if let Err(e) = update_order_res {
            let _ = db_tx.rollback().await;
            return Err(OrderError::Unexpected {
                message: "cannot update order status".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            });
        }

        let commit_res = db_tx.commit().await;
        if let Err(e) = commit_res {
            //TODO: shouldn't we rollback? but how, the commit causes move of db_tx
            //let _ = db_tx.rollback().await;
            return Err(OrderError::Unexpected {
                message: "cannot commit changes to db".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            });
        }
        Ok(())
    }
}
