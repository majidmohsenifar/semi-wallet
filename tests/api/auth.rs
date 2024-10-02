use std::collections::HashMap;

use semi_wallet::{
    handler::response::{ApiError, ApiResponse},
    repository::user::CreateUserArgs,
    service::auth::{bcrypt, service::RegisterResult},
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn register_invalid_inputs() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        (HashMap::new(), "empty email"),
        (
            HashMap::from([
                ("email", "invalid"),
                ("password", "12345678"),
                ("confirm_password", "12345678"),
            ]),
            "invalid email",
        ),
        (
            HashMap::from([("email", "test@test.test")]),
            "empty password",
        ),
        (
            HashMap::from([("email", "test@test.test"), ("password", "1234567")]),
            "password with less than 8 character",
        ),
        (
            HashMap::from([("email", "test@test.test"), ("password", "12345678")]),
            "empty confirm_password",
        ),
        (
            HashMap::from([
                ("email", "test@test.test"),
                ("password", "12345678"),
                ("confirm_password", "123456789"),
            ]),
            "confirm_password not the same as password",
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
            "the api did not fail with 400 Bad Request when the payload has the problem {}",
            msg
        );
    }
}

#[tokio::test]
async fn register_email_already_taken() {
    let app = spawn_app().await;
    let email = "alreadyexist@test.test";
    app.repo
        .create_user(
            &app.db,
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
