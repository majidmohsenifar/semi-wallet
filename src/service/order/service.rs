use sqlx::{Pool, Postgres};

use super::error::OrderError;
use crate::repository::db::Repository;
use crate::repository::models::OrderStatus;
use crate::repository::order::CreateOrderArgs;
use crate::service::payment::service::{CreatePaymentParams, Provider, Service as PaymentService};
use crate::service::plan::service::Service as PlanService;

pub struct Service {
    db: Pool<Postgres>,
    repo: Repository,
    plan_service: PlanService,
    payment_service: PaymentService,
}

#[derive(serde::Deserialize)]
pub struct CreateOrderParams {
    pub plan_id: i64,
    pub user_id: i64,
    pub payment_provider: String,
}

#[derive(serde::Serialize)]
pub struct CreateOrderResult {
    pub order_id: i64,
    pub status: String,
    pub payment_url: String,
    pub payment_provider: String,
}

#[derive(serde::Deserialize)]
pub struct OrderDetailParams {
    pub id: u32,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct OrderDetailResult {
    pub id: u32,
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
        params: CreateOrderParams,
    ) -> Result<CreateOrderResult, OrderError> {
        let payment_provider = Provider::from(&params.payment_provider);
        if payment_provider.is_none() {
            return Err(OrderError::InvalidPaymentProvider);
        }
        let payment_provider = payment_provider.unwrap();
        let plan = self.plan_service.get_plan_by_id(params.plan_id).await;
        if let Err(e) = plan {
            match e {
                sqlx::Error::RowNotFound => {
                    return Err(OrderError::PlanNotFound);
                }
                _ => return Err(OrderError::Unknown),
            }
        }
        let plan = plan.unwrap();

        let db_tx = self.db.begin().await;
        if let Err(e) = db_tx {
            //TODO: log here later
            println!("{e}");
            return Err(OrderError::Unknown);
        }

        let mut db_tx = db_tx.unwrap();

        let order = self
            .repo
            .create_order(
                &mut db_tx,
                CreateOrderArgs {
                    user_id: params.user_id,
                    plan_id: params.plan_id,
                    total: plan.price,
                    status: OrderStatus::Created,
                },
            )
            .await;
        if let Err(e) = order {
            //TODO: we should log here
            println!("{e}");
            return Err(OrderError::Unknown);
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
            //TODO: we should log here
            println!("{e}");
            return Err(OrderError::Unknown);
        }

        let payment = payment.unwrap();
        let tx_res = db_tx.commit().await;
        if let Err(e) = tx_res {
            //TODO: we should log here
            println!("{e}");
            return Err(OrderError::Unknown);
        }
        Ok(CreateOrderResult {
            order_id: order.id,
            status: "CREATED".to_string(), //TODO: handle this later
            payment_url: payment.url,
            payment_provider: params.payment_provider,
        })
    }

    pub async fn order_detail(
        &self,
        params: OrderDetailParams,
    ) -> Result<OrderDetailResult, OrderError> {
        let query_res = sqlx::query("SELECT * from orders where id = $1")
            .fetch_one(&self.db)
            .await;

        let _order = match query_res {
            Err(_) => {
                //TODO: log error and check if it is not found
                return Err(OrderError::NotFound);
            }
            Ok(o) => o,
        };
        Ok(OrderDetailResult { id: params.id })
        //Err(OrderError::NotFound)
    }
}
