use sqlx::{PgConnection, Pool, Postgres};

use crate::repository::{db::Repository, models::User};

use super::error::UserError;

#[derive(Clone)]
pub struct Service {
    db: Pool<Postgres>,
    repo: Repository,
}

#[derive(Debug)]
pub struct CreateUserParams {
    pub email: String,
    pub encrypted_password: String,
}

#[derive(Debug)]
pub struct CreateUserResult {
    pub id: i64,
}

impl Service {
    pub fn new(db: Pool<Postgres>, repo: Repository) -> Self {
        Service { db, repo }
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User, sqlx::Error> {
        self.repo.get_user_by_email(&self.db, &email).await
    }

    pub async fn create_user(
        &self,
        conn: &mut PgConnection,
        params: CreateUserParams,
    ) -> Result<CreateUserResult, UserError> {
        let user = self
            .repo
            .create_user(
                conn,
                crate::repository::user::CreateUserArgs {
                    email: params.email,
                    password: params.encrypted_password,
                },
            )
            .await;

        if let Err(e) = user {
            return Err(UserError::Unexpected {
                message: "cannot create user".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            });
        }
        let user = user.unwrap();
        Ok(CreateUserResult { id: user.id })
    }
}
