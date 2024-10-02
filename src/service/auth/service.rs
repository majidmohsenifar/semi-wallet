use sqlx::{Pool, Postgres};
use validator::Validate;

use crate::repository::{db::Repository, user::CreateUserArgs};

use super::{bcrypt, error::AuthError, jwt};

pub struct Service {
    db: Pool<Postgres>,
    repo: Repository,
    jwt_secret: String,
}

#[derive(serde::Deserialize, Validate)]
pub struct RegisterParams {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 8))]
    pub confirm_password: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct RegisterResult {}

#[derive(serde::Deserialize)]
pub struct LoginParams {
    pub email: String,
    pub password: String,
}

#[derive(serde::Serialize)]
pub struct LoginResult {
    pub token: String,
}

impl Service {
    pub fn new(db: Pool<Postgres>, repo: Repository, jwt_secret: String) -> Self {
        Service {
            db,
            repo,
            jwt_secret,
        }
    }

    pub async fn register(&self, params: RegisterParams) -> Result<RegisterResult, AuthError> {
        let existing_user = self.repo.get_user_by_email(&self.db, &params.email).await;
        match existing_user {
            Err(sqlx::Error::RowNotFound) => {}
            Err(e) => {
                return Err(AuthError::Unexpected {
                    message: "cannot check if email already exist".to_string(),
                    source: Box::new(e) as Box<dyn std::error::Error>,
                });
            }
            Ok(_) => {
                return Err(AuthError::EmailAlreadyTaken);
            }
        }
        let encrypted_password = match bcrypt::encrypt_password(&params.password) {
            Err(e) => {
                return Err(AuthError::Unexpected {
                    message: "cannot encrypt password".to_string(),
                    source: Box::new(e) as Box<dyn std::error::Error>,
                });
            }
            Ok(hashed) => hashed,
        };

        let u = self
            .repo
            .create_user(
                &self.db,
                CreateUserArgs {
                    email: params.email,
                    password: encrypted_password,
                },
            )
            .await;
        if let Err(e) = u {
            return Err(AuthError::Unexpected {
                message: "cannot insert to db".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error>,
            });
        }
        Ok(RegisterResult {})
    }

    pub async fn login(&self, params: LoginParams) -> Result<LoginResult, AuthError> {
        let user = self.repo.get_user_by_email(&self.db, &params.email).await;
        let user = match user {
            Err(sqlx::Error::RowNotFound) => return Err(AuthError::InvalidCredentials),
            Err(e) => {
                return Err(AuthError::Unexpected {
                    message: "something went wrong".to_string(),
                    source: Box::new(e) as Box<dyn std::error::Error>,
                });
            }
            Ok(u) => u,
        };

        let verify = bcrypt::verify_password(&params.password, &user.password);
        let is_verified = match verify {
            Err(e) => {
                return Err(AuthError::Unexpected {
                    message: "cannot verify the password".to_string(),
                    source: Box::new(e) as Box<dyn std::error::Error>,
                });
            }
            Ok(verified) => verified,
        };
        if !is_verified {
            return Err(AuthError::InvalidCredentials);
        }

        let token = jwt::create_jwt(self.jwt_secret.as_bytes(), params.email);
        let token = match token {
            Err(e) => {
                return Err(AuthError::Unexpected {
                    message: "cannot generate token".to_string(),
                    source: Box::new(e) as Box<dyn std::error::Error>,
                });
            }
            Ok(t) => t,
        };

        Ok(LoginResult { token })
    }
}
