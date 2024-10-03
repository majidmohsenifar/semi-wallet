
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
