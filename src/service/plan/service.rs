use sqlx::{Pool, Postgres};

use crate::repository::{db::Repository, models::Plan};

pub struct Service {
    db: Pool<Postgres>,
    repo: Repository,
}

impl Service {
    pub fn new(db: Pool<Postgres>, repo: Repository) -> Self {
        Service { db, repo }
    }
    pub async fn get_plan_by_id(&self, id: i64) -> Result<Plan, sqlx::Error> {
        self.repo.get_plan_by_id(&self.db, id).await
    }
    pub async fn get_plan_by_code(&self, code: &str) -> Result<Plan, sqlx::Error> {
        self.repo.get_plan_by_code(&self.db, code).await
    }
}
