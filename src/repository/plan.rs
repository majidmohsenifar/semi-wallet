use super::{db::Repository, models::Plan};

use sqlx::{Pool, Postgres};

impl Repository {
    pub async fn get_plan_by_id(&self, db: &Pool<Postgres>, id: i64) -> Result<Plan, sqlx::Error> {
        let res = sqlx::query_as::<_, Plan>("SELECT * FROM plans WHERE id = $1")
            .bind(id)
            .fetch_one(db)
            .await?;
        Ok(res)
    }
}
