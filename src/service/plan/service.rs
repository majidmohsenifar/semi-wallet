use bigdecimal::ToPrimitive;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use utoipa::ToSchema;

use crate::repository::{db::Repository, models::Plan as PlanModel};

use super::error::PlanError;

pub const PLAN_CODE_1_MONTH: &str = "1_MONTH";
pub const PLAN_CODE_3_MONTH: &str = "3_MONTH";
pub const PLAN_CODE_6_MONTH: &str = "6_MONTH";
pub const PLAN_CODE_12_MONTH: &str = "12_MONTH";

#[derive(Clone)]
pub struct Service {
    db: Pool<Postgres>,
    repo: Repository,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Plan {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub price: f64,
    pub duration: i16,
    pub save_percentage: i16,
}

impl Service {
    pub fn new(db: Pool<Postgres>, repo: Repository) -> Self {
        Service { db, repo }
    }
    pub async fn get_plan_by_id(&self, id: i64) -> Result<PlanModel, sqlx::Error> {
        self.repo.get_plan_by_id(&self.db, id).await
    }
    pub async fn get_plan_by_code(&self, code: &str) -> Result<PlanModel, sqlx::Error> {
        self.repo.get_plan_by_code(&self.db, code).await
    }

    pub async fn get_plans_list(&self) -> Result<Vec<Plan>, PlanError> {
        let res = self.repo.get_all_plans(&self.db).await;
        if let Err(e) = res {
            return Err(PlanError::Unexpected {
                message: "cannot get plans from db".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            });
        }
        let res = res.unwrap();
        let mut plans = Vec::with_capacity(res.len());
        for p in res {
            let price = match p.price.to_f64() {
                Some(float) => float,
                None => {
                    return Err(PlanError::InvalidPrice);
                }
            };
            plans.push(Plan {
                id: p.id,
                code: p.code,
                name: p.name,
                price,
                duration: p.duration,
                save_percentage: p.save_percentage,
            });
        }
        Ok(plans)
    }
}
