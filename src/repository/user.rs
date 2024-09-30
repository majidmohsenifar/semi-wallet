use sqlx::{Pool, Postgres};

use super::{db::Repository, models::User};

pub struct CreateUserArgs {
    pub email: String,
    pub password: String,
}

impl Repository {
    pub async fn get_user_by_email(
        &self,
        db: &Pool<Postgres>,
        email: &str,
    ) -> Result<User, sqlx::Error> {
        let res = sqlx::query_as::<_, User>("SELECT * from users where email = $1")
            .bind(email)
            .fetch_one(db)
            .await?;
        Ok(res)
    }

    pub async fn create_user(
        &self,
        db: &Pool<Postgres>,
        args: CreateUserArgs,
    ) -> Result<User, sqlx::Error> {
        let res = sqlx::query_as::<_, User>(
            "INSERT INTO users (
                    email ,
                    password ,
                    created_at ,
                    updated_at 
                    ) VALUES (
                    $1, $2, NOW(), NOW()
                    ) RETURNING *",
        )
        .bind(args.email)
        .bind(args.password)
        .fetch_one(db)
        .await?;
        Ok(res)
    }
}
