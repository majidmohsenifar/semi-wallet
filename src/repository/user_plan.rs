use sqlx::{PgConnection, Pool, Postgres};

use super::{db::Repository, models::UserPlan};

pub struct CreateUserArgs {
    pub email: String,
    pub password: String,
}

impl Repository {
    pub async fn get_user_plan_by_user_id(
        &self,
        db: &Pool<Postgres>,
        user_id: i64,
    ) -> Result<UserPlan, sqlx::Error> {
        let res = sqlx::query_as::<_, UserPlan>("SELECT * from users_plans where user_id = $1")
            .bind(user_id)
            .fetch_one(db)
            .await?;
        Ok(res)
    }

    pub async fn create_user_plan_or_update_expires_at(
        &self,
        conn: &mut PgConnection,
        args: CreateUserArgs,
    ) -> Result<UserPlan, sqlx::Error> {
        let res = sqlx::query_as::<_, UserPlan>(
            "INSERT INTO users_plans (
                    user_id,
                    plan_id,
                    expires_at,
                    ) VALUES (
                    $1, $2, $3
                    ) ON CONFLICT (user_id,plan_id) DO UPDATE 
                    SET expires_at =  users_plans.expires_at+EXCLUDED.expires_at, 
                    RETURNING *",
        )
        .bind(args.email)
        .bind(args.password)
        .fetch_one(&mut *conn)
        .await?;
        Ok(res)
    }
}
