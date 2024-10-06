use std::collections::HashMap;

use semi_wallet::{
    handler::response::ApiResponse,
    repository::models::{OrderStatus, PaymentStatus},
    service::{
        order::service::CreateOrderResult, payment::service::PAYMENT_PROVIDER_STRIPE,
        plan::service::PLAN_CODE_1_MONTH,
    },
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn create_order_without_token() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = HashMap::from([
        ("plan_code", PLAN_CODE_1_MONTH),
        ("payment_provider", PAYMENT_PROVIDER_STRIPE),
    ]);
    let response = client
        .post(&format!("{}/api/v1/orders/create", app.address))
        .json(&body)
        .send()
        .await
        .expect("failed to execute request");
    assert_eq!(
        401,
        response.status().as_u16(),
        "the api did not fail with 401 Unauthorized",
    );
}

#[tokio::test]
async fn create_order_invalid_inputs() {
    let app = spawn_app().await;
    //app.get
    let (token, _) = app.get_jwt_token("test@test.com").await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        (HashMap::new(), "empty plan_code"),
        (
            HashMap::from([("plan_code", PLAN_CODE_1_MONTH)]),
            "empty payment provider",
        ),
        (
            HashMap::from([("plan_code", "NOT_EXIST")]),
            "invalid plan_code",
        ),
        (
            HashMap::from([("plan_code", "NOT_EXIST")]),
            "invalid payment provider",
        ),
    ];

    for (body, msg) in test_cases {
        let response = client
            .post(&format!("{}/api/v1/orders/create", app.address))
            .bearer_auth(&token)
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
async fn create_order_1_month_stripe_successful() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let (token, user) = app.get_jwt_token("test@test.com").await;
    let plan_code = PLAN_CODE_1_MONTH;
    let body = HashMap::from([
        ("plan_code", PLAN_CODE_1_MONTH),
        ("payment_provider", PAYMENT_PROVIDER_STRIPE),
    ]);
    let response = client
        .post(&format!("{}/api/v1/orders/create", app.address))
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .expect("failed to execute request");
    assert_eq!(200, response.status().as_u16(), "the api call was not Ok");

    let plan = app.repo.get_plan_by_code(&app.db, plan_code).await.unwrap();
    let bytes = response.bytes().await.unwrap();
    let res: ApiResponse<'_, CreateOrderResult> = serde_json::from_slice(&bytes).unwrap();
    let data = res.data.unwrap();
    assert_eq!(&data.status, "CREATED");
    assert_ne!(&data.payment_url, "");

    let mut conn = app.db.acquire().await.unwrap();
    let order = app.repo.get_order_by_id(&mut conn, data.id).await.unwrap();
    assert_eq!(order.id, data.id);
    assert_eq!(order.user_id, user.id);
    assert_eq!(order.status, OrderStatus::Created);
    assert_eq!(order.plan_id, plan.id);
    assert_eq!(order.total, plan.price);

    let payment = app
        .repo
        .get_last_payment_by_order_id(&app.db, order.id)
        .await
        .unwrap();

    assert_eq!(payment.user_id, user.id);
    assert_eq!(payment.status, PaymentStatus::Created);
    assert_eq!(payment.amount, order.total);
    assert_eq!(payment.payment_provider_code, PAYMENT_PROVIDER_STRIPE);
    assert_ne!(payment.external_id, None);
}
