use sqlx::{PgConnection, Pool, Postgres};

use super::{db::Repository, models::UserPlan};

#[derive(Debug)]
pub struct CreateUserPlanOrUpdateExpiresAtArgs {
    pub user_id: i64,
    pub plan_id: i64,
    pub order_id: i64,
    pub days: i16,
}

#[derive(Debug, sqlx::FromRow)]
pub struct GetNonExpiredUsersPlansRow {
    pub id: i64,
    pub user_id: i64,
}

impl Repository {
    pub async fn get_user_plan_by_user_id(
        &self,
        db: &Pool<Postgres>,
        user_id: i64,
    ) -> Result<UserPlan, sqlx::Error> {
        let res = sqlx::query_as::<_, UserPlan>("SELECT * from users_plans WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(db)
            .await?;
        Ok(res)
    }

    pub async fn create_user_plan_or_update_expires_at(
        &self,
        conn: &mut PgConnection,
        args: CreateUserPlanOrUpdateExpiresAtArgs,
    ) -> Result<UserPlan, sqlx::Error> {
        //expires_at = users_plans.expires_at + (interval '1 day' * $4),
        let res = sqlx::query_as::<_, UserPlan>(
            "INSERT INTO users_plans (
        user_id,
        last_plan_id,
        last_order_id,
        expires_at
        ) VALUES (
        $1, $2, $3, NOW() + (interval '1 day' * $4)
        ) ON CONFLICT (user_id) DO UPDATE
        SET 
        expires_at = CASE WHEN (users_plans.expires_at > NOW()) THEN users_plans.expires_at + (interval '1 day' * $4) ELSE NOW() + (interval '1 day' * $4) END, 
        last_plan_id = EXCLUDED.last_plan_id,
        last_order_id = EXCLUDED.last_order_id
        RETURNING *",
        )
        .bind(args.user_id)
        .bind(args.plan_id)
        .bind(args.order_id)
        .bind(args.days)
        .fetch_one(&mut *conn)
        .await?;
        Ok(res)
    }

    pub async fn get_non_expired_users_plans(
        &self,
        db: &Pool<Postgres>,
        last_id: i64,
        page_size: i64,
    ) -> Result<Vec<GetNonExpiredUsersPlansRow>, sqlx::Error> {
        let res = sqlx::query_as::<_, GetNonExpiredUsersPlansRow>(
            "SELECT id, user_id from users_plans WHERE expires_at >= NOW() AND id > $1 ORDER BY id ASC LIMIT $2",
        )
        .bind(last_id)
        .bind(page_size)
        .fetch_all(db)
        .await?;
        Ok(res)
    }
}
