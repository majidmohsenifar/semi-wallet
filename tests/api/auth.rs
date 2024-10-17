use std::collections::HashMap;

use semi_wallet::{
    handler::response::{ApiError, ApiResponse},
    repository::user::CreateUserArgs,
    service::auth::{
        bcrypt,
        service::{LoginResult, RegisterResult},
    },
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn register_invalid_inputs() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        (HashMap::new(), "missing field `email`"),
        (
            HashMap::from([
                ("email", "invalid"),
                ("password", "12345678"),
                ("confirm_password", "12345678"),
            ]),
            "email: not valid",
        ),
        (
            HashMap::from([("email", "test@test.test")]),
            "missing field `password`",
        ),
        (
            HashMap::from([
                ("email", "test@test.test"),
                ("password", "1234567"),
                ("confirm_password", "12345678"),
            ]),
            "password: must be at least 8 characters",
        ),
        (
            HashMap::from([("email", "test@test.test"), ("password", "12345678")]),
            "missing field `confirm_password`",
        ),
        (
            HashMap::from([
                ("email", "test@test.test"),
                ("password", "12345678"),
                ("confirm_password", "1234567"),
            ]),
            "confirm_password: must be at least 8 characters",
        ),
        (
            HashMap::from([
                ("email", "test@test.test"),
                ("password", "12345678"),
                ("confirm_password", "123456789"),
            ]),
            "confirm_password is not the same as password",
        ),
    ];

    for (body, msg) in test_cases {
        let response = client
            .post(&format!("{}/api/v1/auth/register", app.address))
            .json(&body)
            .send()
            .await
            .expect("failed to execute request");
        assert_eq!(
            400,
            response.status().as_u16(),
            "the api did not fail with 400 Bad Request",
        );
        let bytes = response.bytes().await.unwrap();
        let res: ApiError<'_> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(res.message, msg);
    }
}

#[tokio::test]
async fn register_email_already_taken() {
    let app = spawn_app().await;
    let email = "alreadyexist@test.test";

    let mut conn = app.db.acquire().await.unwrap();
    app.repo
        .create_user(
            &mut conn,
            CreateUserArgs {
                email: email.to_string(),
                password: "123456789".to_string(),
            },
        )
        .await
        .unwrap();

    let body = HashMap::from([
        ("email", email),
        ("password", "12345678"),
        ("confirm_password", "12345678"),
    ]);
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/api/v1/auth/register", app.address))
        .json(&body)
        .send()
        .await
        .expect("failed to execute request");
    assert_eq!(
        422,
        response.status().as_u16(),
        "user already exxist we should return 422",
    );
    let bytes = response.bytes().await.unwrap();
    let res: ApiError<'_> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(res.message, "email already exist");
}

#[tokio::test]
async fn register_successful() {
    let app = spawn_app().await;
    let email = "success@test.test";
    let body = HashMap::from([
        ("email", email),
        ("password", "12345678"),
        ("confirm_password", "12345678"),
    ]);
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/api/v1/auth/register", app.address))
        .json(&body)
        .send()
        .await
        .expect("failed to execute request");
    assert_eq!(200, response.status().as_u16());
    let bytes = response.bytes().await.unwrap();
    let res: ApiResponse<'_, RegisterResult> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(res.message, "");

    // check the existance of user in db
    let u = app.repo.get_user_by_email(&app.db, email).await.unwrap();
    let is_verified = bcrypt::verify_password("12345678", &u.password).unwrap();
    assert!(is_verified);
}

#[tokio::test]
async fn login_invalid_inputs() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        (HashMap::new(), "missing field `email`"),
        (
            HashMap::from([("email", "invalid"), ("password", "12345678")]),
            "email: not valid",
        ),
        (
            HashMap::from([("email", "test@test.test")]),
            "missing field `password`",
        ),
        (
            HashMap::from([("email", "test@test.test"), ("password", "1234567")]),
            "password: must be at least 8 characters",
        ),
    ];

    for (body, msg) in test_cases {
        let response = client
            .post(&format!("{}/api/v1/auth/login", app.address))
            .json(&body)
            .send()
            .await
            .expect("failed to execute request");
        assert_eq!(
            400,
            response.status().as_u16(),
            "the api did not fail with 400 Bad Request when the payload has the problem {}",
            msg
        );
        let bytes = response.bytes().await.unwrap();
        let res: ApiError<'_> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(res.message, msg);
    }
}

#[tokio::test]
async fn login_invalid_credential_email_does_not_exist() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let email = "doesnotexist@test.test";
    let body = HashMap::from([("email", email), ("password", "12345678")]);
    let response = client
        .post(&format!("{}/api/v1/auth/login", app.address))
        .json(&body)
        .send()
        .await
        .expect("failed to execute request");
    assert_eq!(
        401,
        response.status().as_u16(),
        "should get the error as unauthorized",
    );
}

#[tokio::test]
async fn login_invalid_credential_wrong_password() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let email = "wrongpassword@test.test";

    let encrypted_password = bcrypt::encrypt_password("12345678").unwrap();
    let mut conn = app.db.acquire().await.unwrap();
    app.repo
        .create_user(
            &mut conn,
            CreateUserArgs {
                email: email.to_string(),
                password: encrypted_password,
            },
        )
        .await
        .unwrap();

    let body = HashMap::from([("email", email), ("password", "wrong")]);
    let response = client
        .post(&format!("{}/api/v1/auth/login", app.address))
        .json(&body)
        .send()
        .await
        .expect("failed to execute request");
    assert_eq!(
        401,
        response.status().as_u16(),
        "should get the error as unauthorized because the password is worng",
    );
}

#[tokio::test]
async fn login_successful() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let email = "success@test.test";

    let encrypted_password = bcrypt::encrypt_password("12345678").unwrap();
    let mut conn = app.db.acquire().await.unwrap();
    app.repo
        .create_user(
            &mut conn,
            CreateUserArgs {
                email: email.to_string(),
                password: encrypted_password,
            },
        )
        .await
        .unwrap();

    let body = HashMap::from([("email", email), ("password", "12345678")]);
    let response = client
        .post(&format!("{}/api/v1/auth/login", app.address))
        .json(&body)
        .send()
        .await
        .expect("failed to execute request");
    assert_eq!(200, response.status().as_u16());

    let bytes = response.bytes().await.unwrap();
    let res: ApiResponse<'_, LoginResult> = serde_json::from_slice(&bytes).unwrap();
    let data = res.data.unwrap();
    assert_ne!(data.token, "");
    assert_eq!(res.message, "");
}
