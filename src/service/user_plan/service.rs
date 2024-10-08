use sqlx::{Pool, Postgres};

use crate::repository::{
    db::Repository, models::Plan, user_plan::CreateUserPlanOrUpdateExpiresAtArgs,
};

use super::error::UserPlanError;

pub struct Service {
    db: Pool<Postgres>,
    repo: Repository,
}

impl Service {
    pub fn new(db: Pool<Postgres>, repo: Repository) -> Self {
        Service { db, repo }
    }

    pub async fn create_user_plan_or_update_expires_at(
        &self,
        db_tx: &mut sqlx::Transaction<'_, Postgres>,
        user_id: i64,
        plan: Plan,
        order_id: i64,
    ) -> Result<(), UserPlanError> {
        let res = self
            .repo
            .create_user_plan_or_update_expires_at(
                db_tx,
                CreateUserPlanOrUpdateExpiresAtArgs {
                    user_id,
                    plan_id: plan.id,
                    order_id,
                    days: plan.duration + 1, //adding 1 just to be sure that it would be more than
                                             //plan duration
                },
            )
            .await;

        if let Err(e) = res {
            return Err(UserPlanError::Unexpected {
                message: "cannot create or update user_plan".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            });
        }
        Ok(())
    }
}
