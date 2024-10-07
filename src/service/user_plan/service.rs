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
    ) -> Result<(), UserPlanError> {
        let expires_at =
            chrono::Utc::now().checked_add_days(chrono::Days::new(plan.duration as u64));
        let expires_at = match expires_at {
            Some(ex) => ex,
            None => {
                return Err(UserPlanError::InvalidExpiration);
            }
        };

        let res = self
            .repo
            .create_user_plan_or_update_expires_at(
                db_tx,
                CreateUserPlanOrUpdateExpiresAtArgs {
                    user_id,
                    plan_id: plan.id,
                    expires_at,
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
