use std::collections::HashMap;

use bigdecimal::{BigDecimal, FromPrimitive};
use claims::assert_gt;
use std::str::FromStr;
use stripe::{CheckoutSession, CheckoutSessionId, CheckoutSessionStatus};
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use semi_wallet::{
    handler::api::response::{ApiError, ApiResponse},
    repository::{
        models::{OrderStatus, PaymentStatus},
        order::CreateOrderArgs,
        payment::CreatePaymentArgs,
    },
    service::{
        order::service::{CreateOrderResult, Order, OrderDetailResult},
        payment::service::PAYMENT_PROVIDER_STRIPE,
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
    let (token, _) = app.get_jwt_token_and_user("test@test.com").await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        (HashMap::new(), "missing field `plan_code`"),
        (
            HashMap::from([("plan_code", PLAN_CODE_1_MONTH)]),
            "missing field `payment_provider`",
        ),
        (
            HashMap::from([
                ("payment_provider", "WRONG"),
                ("plan_code", PLAN_CODE_1_MONTH),
            ]),
            "payment_provider: not valid",
        ),
        (
            HashMap::from([
                ("payment_provider", PAYMENT_PROVIDER_STRIPE),
                ("plan_code", "WRONG"),
            ]),
            "plan_code: not valid",
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
            msg,
        );
        let bytes = response.bytes().await.unwrap();
        let res: ApiError<'_> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(res.message, msg);
    }
}

#[tokio::test]
async fn create_order_1_month_stripe_successful() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let (token, user) = app.get_jwt_token_and_user("test@test.com").await;
    let plan_code = PLAN_CODE_1_MONTH;
    let body = HashMap::from([
        ("plan_code", PLAN_CODE_1_MONTH),
        ("payment_provider", PAYMENT_PROVIDER_STRIPE),
    ]);
    let stripe_payment_url = "https://test.test".to_string();

    let checkout_session = CheckoutSession {
        id: CheckoutSessionId::from_str(
            "cs_test_a11YYufWQzNY63zpQ6QSNRQhkUpVph4WRmzW0zWJO2znZKdVujZ0N0S22u",
        )
        .unwrap(),
        status: Some(CheckoutSessionStatus::Open),
        url: Some(stripe_payment_url.clone()),
        ..Default::default()
    };

    Mock::given(path("/v1/checkout/sessions"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&checkout_session))
        .expect(1)
        .mount(&app.stripe_server)
        .await;

    let response = client
        .post(&format!("{}/api/v1/orders/create", app.address))
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .expect("failed to execute request");
    assert_eq!(200, response.status().as_u16(), "the api call was not Ok",);

    let plan = app.repo.get_plan_by_code(&app.db, plan_code).await.unwrap();
    let bytes = response.bytes().await.unwrap();
    let res: ApiResponse<'_, CreateOrderResult> = serde_json::from_slice(&bytes).unwrap();
    let data = res.data.unwrap();
    assert_eq!(&data.status, "CREATED");
    assert_eq!(&data.payment_url, &stripe_payment_url);

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

    app.stripe_server.verify().await;
}

#[tokio::test]
async fn order_detail_invalid_inputs() {
    //we do not provide id in the url
    let app = spawn_app().await;
    let (token, _) = app.get_jwt_token_and_user("test@test.test").await;
    let test_cases: Vec<(&[(&str, &str)], &str)> = vec![
        (&[], "id is required and must be u64"),
        (&[("id", "wrong")], "id is required and must be u64"),
    ];

    let client = reqwest::Client::new();
    for (q, msg) in test_cases {
        let response = client
            .get(format!("{}/api/v1/orders/detail", app.address))
            .bearer_auth(&token)
            .query(q)
            .send()
            .await
            .expect("failed to call api");

        assert_eq!(
            400,
            response.status().as_u16(),
            "api did not return 400 Bad Request"
        );
        let bytes = response.bytes().await.unwrap();
        let res: ApiError<'_> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(res.message, msg);
    }
}

#[tokio::test]
async fn order_detail_status_created_successful() {
    let app = spawn_app().await;

    let (token, user) = app.get_jwt_token_and_user("test@test.test").await;
    //create order and payment
    let plan = app
        .repo
        .get_plan_by_code(&app.db, PLAN_CODE_1_MONTH)
        .await
        .unwrap();
    let mut conn = app.db.acquire().await.unwrap();
    let order = app
        .repo
        .create_order(
            &mut conn,
            semi_wallet::repository::order::CreateOrderArgs {
                user_id: user.id,
                plan_id: plan.id,
                total: BigDecimal::from_f64(1.99).unwrap(),
                status: OrderStatus::Created,
            },
        )
        .await
        .unwrap();
    let payment = app
        .repo
        .create_payment(
            &mut conn,
            CreatePaymentArgs {
                user_id: user.id,
                order_id: order.id,
                payment_provider_code: PAYMENT_PROVIDER_STRIPE.to_string(),
                amount: BigDecimal::from_f64(1.99).unwrap(),
                status: PaymentStatus::Created,
            },
        )
        .await
        .unwrap();

    app.repo
        .update_payment_external_id_payment_url_expires_at(
            &mut conn,
            payment.id,
            "stripe_id",
            "https://stripe.test.test",
            chrono::Utc::now(),
        )
        .await
        .unwrap();

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v1/orders/detail", app.address))
        .query(&[("id", &order.id.to_string())])
        .bearer_auth(&token)
        .send()
        .await
        .expect("failed to call api");

    assert_eq!(200, response.status().as_u16(), "api did not return 200 Ok");

    let bytes = response.bytes().await.unwrap();
    let res: ApiResponse<'_, OrderDetailResult> = serde_json::from_slice(&bytes).unwrap();
    let data = res.data.unwrap();

    assert_eq!(data.id, order.id);
    assert_eq!(data.plan_code, PLAN_CODE_1_MONTH);
    assert_eq!(data.total, 1.99);
    assert_eq!(data.status, "CREATED");
    assert_eq!(data.payment_url, "https://stripe.test.test");
    assert_gt!(data.payment_expire_date, 0);
}

#[tokio::test]
async fn order_detail_status_completed_successful() {
    let app = spawn_app().await;

    let (token, user) = app.get_jwt_token_and_user("test@test.test").await;
    //create order and payment
    let plan = app
        .repo
        .get_plan_by_code(&app.db, PLAN_CODE_1_MONTH)
        .await
        .unwrap();
    let mut conn = app.db.acquire().await.unwrap();
    let order = app
        .repo
        .create_order(
            &mut conn,
            semi_wallet::repository::order::CreateOrderArgs {
                user_id: user.id,
                plan_id: plan.id,
                total: BigDecimal::from_f64(1.99).unwrap(),
                status: OrderStatus::Completed,
            },
        )
        .await
        .unwrap();
    let payment = app
        .repo
        .create_payment(
            &mut conn,
            CreatePaymentArgs {
                user_id: user.id,
                order_id: order.id,
                payment_provider_code: PAYMENT_PROVIDER_STRIPE.to_string(),
                amount: BigDecimal::from_f64(1.99).unwrap(),
                status: PaymentStatus::Completed,
            },
        )
        .await
        .unwrap();

    app.repo
        .update_payment_external_id_payment_url_expires_at(
            &mut conn,
            payment.id,
            "stripe_id",
            "https://stripe.test.test",
            chrono::Utc::now(),
        )
        .await
        .unwrap();

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v1/orders/detail", app.address))
        .query(&[("id", &order.id.to_string())])
        .bearer_auth(&token)
        .send()
        .await
        .expect("failed to call api");

    assert_eq!(200, response.status().as_u16(), "api did not return 200 Ok");

    let bytes = response.bytes().await.unwrap();
    let res: ApiResponse<'_, OrderDetailResult> = serde_json::from_slice(&bytes).unwrap();
    let data = res.data.unwrap();

    assert_eq!(data.id, order.id);
    assert_eq!(data.plan_code, PLAN_CODE_1_MONTH);
    assert_eq!(data.total, 1.99);
    assert_eq!(data.status, "COMPLETED");
    assert_eq!(data.payment_url, "");
    assert_gt!(data.payment_expire_date, 0);
}

#[tokio::test]
async fn order_detail_status_failed_successful() {
    let app = spawn_app().await;

    let (token, user) = app.get_jwt_token_and_user("test@test.test").await;
    //create order and payment
    let plan = app
        .repo
        .get_plan_by_code(&app.db, PLAN_CODE_1_MONTH)
        .await
        .unwrap();
    let mut conn = app.db.acquire().await.unwrap();
    let order = app
        .repo
        .create_order(
            &mut conn,
            semi_wallet::repository::order::CreateOrderArgs {
                user_id: user.id,
                plan_id: plan.id,
                total: BigDecimal::from_f64(1.99).unwrap(),
                status: OrderStatus::Failed,
            },
        )
        .await
        .unwrap();
    let payment = app
        .repo
        .create_payment(
            &mut conn,
            CreatePaymentArgs {
                user_id: user.id,
                order_id: order.id,
                payment_provider_code: PAYMENT_PROVIDER_STRIPE.to_string(),
                amount: BigDecimal::from_f64(1.99).unwrap(),
                status: PaymentStatus::Failed,
            },
        )
        .await
        .unwrap();

    app.repo
        .update_payment_external_id_payment_url_expires_at(
            &mut conn,
            payment.id,
            "stripe_id",
            "https://stripe.test.test",
            chrono::Utc::now(),
        )
        .await
        .unwrap();

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/v1/orders/detail", app.address))
        .query(&[("id", &order.id.to_string())])
        .bearer_auth(&token)
        .send()
        .await
        .expect("failed to call api");

    assert_eq!(200, response.status().as_u16(), "api did not return 200 Ok");

    let bytes = response.bytes().await.unwrap();
    let res: ApiResponse<'_, OrderDetailResult> = serde_json::from_slice(&bytes).unwrap();
    let data = res.data.unwrap();

    assert_eq!(data.id, order.id);
    assert_eq!(data.plan_code, PLAN_CODE_1_MONTH);
    assert_eq!(data.total, 1.99);
    assert_eq!(data.status, "FAILED");
    assert_eq!(data.payment_url, "");
    assert_gt!(data.payment_expire_date, 0);
}

#[tokio::test]
async fn user_orders_without_token() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/api/v1/orders", app.address))
        .send()
        .await
        .expect("failed to call api");

    assert_eq!(
        401,
        response.status().as_u16(),
        "the api did not fail with 401 Unauthorized",
    );
}

#[tokio::test]
async fn user_orders_list_successful() {
    let app = spawn_app().await;

    let (token, user) = app.get_jwt_token_and_user("test@test.test").await;

    let plan = app
        .repo
        .get_plan_by_code(&app.db, PLAN_CODE_1_MONTH)
        .await
        .unwrap();

    let mut conn = app.db.acquire().await.unwrap();
    let o1 = app
        .repo
        .create_order(
            &mut conn,
            CreateOrderArgs {
                user_id: user.id,
                plan_id: plan.id,
                total: BigDecimal::from_f64(1.99).unwrap(),
                status: OrderStatus::Completed,
            },
        )
        .await
        .unwrap();
    let o2 = app
        .repo
        .create_order(
            &mut conn,
            CreateOrderArgs {
                user_id: user.id,
                plan_id: plan.id,
                total: BigDecimal::from_f64(2.99).unwrap(),
                status: OrderStatus::Completed,
            },
        )
        .await
        .unwrap();
    let o3 = app
        .repo
        .create_order(
            &mut conn,
            CreateOrderArgs {
                user_id: user.id,
                plan_id: plan.id,
                total: BigDecimal::from_f64(1.99).unwrap(),
                status: OrderStatus::Failed,
            },
        )
        .await
        .unwrap();
    let o4 = app
        .repo
        .create_order(
            &mut conn,
            CreateOrderArgs {
                user_id: user.id,
                plan_id: plan.id,
                total: BigDecimal::from_f64(1.99).unwrap(),
                status: OrderStatus::Created,
            },
        )
        .await
        .unwrap();

    let client = reqwest::Client::new();

    //for page 0
    let response = client
        .get(format!("{}/api/v1/orders", app.address))
        .bearer_auth(&token)
        .query(&[("page", &0.to_string()), ("page_size", &2.to_string())])
        .send()
        .await
        .expect("failed to call api");

    assert_eq!(200, response.status().as_u16(), "the api call was not Ok",);

    let bytes = response.bytes().await.unwrap();
    let res: ApiResponse<'_, Vec<Order>> = serde_json::from_slice(&bytes).unwrap();
    let data = res.data.unwrap();
    assert_eq!(data.len(), 2);
    let order4 = data.first().unwrap();
    assert_eq!(order4.id, o4.id);
    assert_eq!(order4.plan_id, plan.id);
    assert_eq!(order4.total, 1.99);
    assert_eq!(order4.status, "CREATED".to_string());
    assert_eq!(order4.created_at, o4.created_at.timestamp());

    let order3 = data.get(1).unwrap();
    assert_eq!(order3.id, o3.id);
    assert_eq!(order3.plan_id, plan.id);
    assert_eq!(order3.total, 1.99);
    assert_eq!(order3.status, "FAILED".to_string());
    assert_eq!(order3.created_at, o3.created_at.timestamp());

    //for page 1
    let response = client
        .get(format!("{}/api/v1/orders", app.address))
        .bearer_auth(&token)
        .query(&[("page", &1.to_string()), ("page_size", &2.to_string())])
        .send()
        .await
        .expect("failed to call api");

    assert_eq!(200, response.status().as_u16(), "the api call was not Ok",);

    let bytes = response.bytes().await.unwrap();
    let res: ApiResponse<'_, Vec<Order>> = serde_json::from_slice(&bytes).unwrap();
    let data = res.data.unwrap();
    assert_eq!(data.len(), 2);
    let order2 = data.first().unwrap();
    assert_eq!(order2.id, o2.id);
    assert_eq!(order2.plan_id, plan.id);
    assert_eq!(order2.total, 2.99);
    assert_eq!(order2.status, "COMPLETED".to_string());
    assert_eq!(order2.created_at, o2.created_at.timestamp());

    let order1 = data.get(1).unwrap();
    assert_eq!(order1.id, o1.id);
    assert_eq!(order1.plan_id, plan.id);
    assert_eq!(order1.total, 1.99);
    assert_eq!(order1.status, "COMPLETED".to_string());
    assert_eq!(order1.created_at, o1.created_at.timestamp());

    //without sending any page and page_size
    let response = client
        .get(format!("{}/api/v1/orders", app.address))
        .bearer_auth(&token)
        .send()
        .await
        .expect("failed to call api");

    assert_eq!(200, response.status().as_u16(), "the api call was not Ok",);

    let bytes = response.bytes().await.unwrap();
    let res: ApiResponse<'_, Vec<Order>> = serde_json::from_slice(&bytes).unwrap();
    let data = res.data.unwrap();
    assert_eq!(data.len(), 4);

    //sending negative page and page_size
    let response = client
        .get(format!("{}/api/v1/orders", app.address))
        .bearer_auth(&token)
        .query(&[
            ("page", &(-1).to_string()),
            ("page_size", &(-2).to_string()),
        ])
        .send()
        .await
        .expect("failed to call api");

    assert_eq!(200, response.status().as_u16(), "the api call was not Ok",);

    let bytes = response.bytes().await.unwrap();
    let res: ApiResponse<'_, Vec<Order>> = serde_json::from_slice(&bytes).unwrap();
    let data = res.data.unwrap();
    assert_eq!(data.len(), 4);
}
