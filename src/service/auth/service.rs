use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use utoipa::ToSchema;
use validator::Validate;

use crate::repository::models::User;
use crate::service::user::error::UserError;
use crate::service::user::service::{CreateUserParams, Service as UserService};

use super::{bcrypt, error::AuthError, jwt};

#[derive(Clone)]
pub struct Service {
    db: Pool<Postgres>,
    user_service: UserService,
    jwt_secret: String,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct RegisterParams<'a> {
    #[validate(email(message = "not valid"))]
    pub email: &'a str,
    #[validate(length(min = 8, message = "must be at least 8 characters"))]
    pub password: &'a str,
    #[validate(length(min = 8, message = "must be at least 8 characters"))]
    pub confirm_password: &'a str,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RegisterResult {}

#[derive(serde::Deserialize, Validate, ToSchema)]
pub struct LoginParams {
    #[validate(email(message = "not valid"))]
    pub email: String,
    #[validate(length(min = 8, message = "must be at least 8 characters"))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginResult {
    pub token: String,
}

impl Service {
    pub fn new(db: Pool<Postgres>, user_service: UserService, jwt_secret: String) -> Self {
        Service {
            db,
            user_service,
            jwt_secret,
        }
    }

    pub async fn register(&self, params: RegisterParams<'_>) -> Result<RegisterResult, AuthError> {
        let existing_user = self.user_service.get_user_by_email(params.email).await;
        match existing_user {
            Err(sqlx::Error::RowNotFound) => {}
            Err(e) => {
                tracing::error!("cannot get_user_by_email due to err: {}", e);
                return Err(AuthError::Unexpected {
                    message: "cannot check if email already exist".to_string(),
                    source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
                });
            }
            Ok(_) => {
                return Err(AuthError::EmailAlreadyTaken);
            }
        }
        let encrypted_password = match bcrypt::encrypt_password(params.password) {
            Err(e) => {
                tracing::error!("cannot encrypt password due to err: {}", e);
                return Err(AuthError::Unexpected {
                    message: "cannot encrypt password".to_string(),
                    source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
                });
            }
            Ok(hashed) => hashed,
        };

        let mut conn = self.db.acquire().await.map_err(|e| {
            tracing::error!("cannot acquire db conn due to err: {}", e);
            AuthError::Unexpected {
                message: "cannot acquire db conn ".to_string(),
                source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
            }
        })?;
        let create_user_res = self
            .user_service
            .create_user(
                &mut conn,
                CreateUserParams {
                    email: params.email,
                    encrypted_password: &encrypted_password,
                },
            )
            .await;
        match create_user_res {
            Ok(_) => {}
            Err(UserError::Unexpected { source, .. }) => {
                tracing::error!("cannot create_user due to err: {}", source);
                return Err(AuthError::Unexpected {
                    message: "cannot insert to db".to_string(),
                    source,
                });
            }
        }
        Ok(RegisterResult {})
    }

    pub async fn login(&self, params: LoginParams) -> Result<LoginResult, AuthError> {
        let user = self.user_service.get_user_by_email(&params.email).await;
        let user = match user {
            Err(sqlx::Error::RowNotFound) => return Err(AuthError::InvalidCredentials),
            Err(e) => {
                tracing::error!("cannot get user by email due to err: {}", e);
                return Err(AuthError::Unexpected {
                    message: "something went wrong".to_string(),
                    source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
                });
            }
            Ok(u) => u,
        };

        let verify = bcrypt::verify_password(&params.password, &user.password);
        let is_verified = match verify {
            Err(e) => {
                tracing::error!("cannot verify_password due to err: {}", e);
                return Err(AuthError::Unexpected {
                    message: "cannot verify the password".to_string(),
                    source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
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
                tracing::error!("cannot create_jwt due to err: {}", e);
                return Err(AuthError::Unexpected {
                    message: "cannot generate token".to_string(),
                    source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
                });
            }
            Ok(t) => t,
        };

        Ok(LoginResult { token })
    }

    pub async fn get_user_from_token(&self, token: &str) -> Result<User, AuthError> {
        let email = jwt::get_email_from_token(self.jwt_secret.as_bytes(), token);
        let email = match email {
            Err(_e) => {
                return Err(AuthError::InvalidToken);
            }
            Ok(t) => t,
        };
        let user = self.user_service.get_user_by_email(&email).await;
        let user = match user {
            Err(sqlx::Error::RowNotFound) => return Err(AuthError::InvalidToken),
            Err(e) => {
                tracing::error!("cannot get_user_by_email due to err: {}", e);
                return Err(AuthError::Unexpected {
                    message: "something went wrong".to_string(),
                    source: Box::new(e) as Box<dyn std::error::Error + Send + Sync>,
                });
            }
            Ok(u) => u,
        };
        Ok(user)
    }
}
