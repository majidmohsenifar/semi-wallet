use std::collections::HashMap;

use crate::helpers::spawn_app;

#[tokio::test]
async fn register_invalid_inputs() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        //(HashMap::new(), "empty email"),
        (
            HashMap::from([
                ("email", "invalid"),
                ("password", "12345678"),
                ("confirm_password", "12345678"),
            ]),
            "invalid email",
        ),
        //(
        //HashMap::from([("email", "test@test.test")]),
        //"empty password",
        //),
        //(
        //HashMap::from([("email", "test@test.test"), ("password", "1234567")]),
        //"password with less than 8 character",
        //),
        //(
        //HashMap::from([("email", "test@test.test"), ("password", "12345678")]),
        //"empty confirm_password",
        //),
        //(
        //HashMap::from([
        //("email", "test@test.test"),
        //("password", "12345678"),
        //("confirm_password", "123456789"),
        //]),
        //"confirm_password not the same as password",
        //),
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
