use sqlx::{Pool, Postgres};

use super::error::OrderError;

use crate::service::payment::service::{CreatePaymentParams, Service as PaymentService};

pub struct Service {
    db: Pool<Postgres>,
    payment_service: PaymentService,
}

pub struct CreateOrderParams {
    pub plan_id: u32,
}

pub struct CreateOrderResult {
    pub id: u32,
}

#[derive(Debug, serde::Deserialize)]
pub struct OrderDetailParams {
    pub id: u32,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct OrderDetailResult {
    pub id: u32,
}

impl Service {
    pub fn new(db: Pool<Postgres>, payment_service: PaymentService) -> Self {
        Service {
            db,
            payment_service,
        }
    }

    //pub fn create_order(
    //&self,
    //params: CreateOrderParams,
    //) -> Result<CreateOrderResult, Box<dyn Error>> {
    //println!("plan_id is {}", params.plan_id);
    //Ok(CreateOrderResult { id: 1 })
    //}

    pub async fn order_detail(
        &self,
        params: OrderDetailParams,
    ) -> Result<OrderDetailResult, OrderError> {
        let query_res = sqlx::query("SELECT * from orders where id = ?")
            .fetch_one(&self.db)
            .await;

        let order = match query_res {
            Err(e) => {
                return Err(OrderError::NotFound);
            }
            Ok(o) => o,
        };
        Ok(OrderDetailResult { id: params.id })
        //Err(OrderError::NotFound)
    }
}
