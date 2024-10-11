use bigdecimal::ToPrimitive;
use claim::assert_gt;
use std::str::FromStr;

use stripe::{
    CheckoutSession, CheckoutSessionId, CheckoutSessionStatus, EventId, EventObject, EventType,
    NotificationEventData,
};

use hmac::{Hmac, Mac};

use semi_wallet::{
    repository::{
        models::{OrderStatus, PaymentStatus},
        order::CreateOrderArgs,
        payment::CreatePaymentArgs,
        user_plan::CreateUserPlanOrUpdateExpiresAtArgs,
    },
    service::{
        payment::service::PAYMENT_PROVIDER_STRIPE,
        plan::service::{PLAN_CODE_1_MONTH, PLAN_CODE_3_MONTH},
    },
};
use sha2::Sha256;
use stripe::Event;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn stripe_webhook_still_created_first_time() {
    let app = spawn_app().await;

    let mut conn = app.db.acquire().await.unwrap();

    let (_, user) = app.get_jwt_token_and_user("test@test.test").await;

    let plan = app
        .repo
        .get_plan_by_code(&app.db, PLAN_CODE_1_MONTH)
        .await
        .unwrap();

    let order = app
        .repo
        .create_order(
            &mut conn,
            CreateOrderArgs {
                user_id: user.id,
                plan_id: plan.id,
                total: plan.price,
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
                amount: order.total,
                status: PaymentStatus::Created,
            },
        )
        .await
        .unwrap();

    app.repo
        .update_payment_external_id(
            &mut conn,
            payment.id,
            "cs_test_a11YYufWQzNY63zpQ6QSNRQhkUpVph4WRmzW0zWJO2znZKdVujZ0N0S22u",
        )
        .await
        .unwrap();

    let checkout_session = CheckoutSession {
        id: CheckoutSessionId::from_str(
            "cs_test_a11YYufWQzNY63zpQ6QSNRQhkUpVph4WRmzW0zWJO2znZKdVujZ0N0S22u",
        )
        .unwrap(),
        amount_total: Some(payment.amount.to_i64().unwrap()),
        status: Some(CheckoutSessionStatus::Open),
        client_reference_id: Some(payment.id.to_string()),
        ..Default::default()
    };

    let e_object = EventObject::CheckoutSession(checkout_session.clone());
    let mut notif_event_data = NotificationEventData {
        ..Default::default()
    };
    notif_event_data.object = e_object; //TODO: why this works, but we cannot set it in the struct
                                        //itself
    let event = Event {
        id: EventId::from_str("evt_e").unwrap(),
        type_: EventType::CheckoutSessionCompleted,
        data: notif_event_data,
        ..Default::default()
    };
    let data = serde_json::to_string(&event).unwrap();

    Mock::given(path(
        "/v1/checkout/sessions/cs_test_a11YYufWQzNY63zpQ6QSNRQhkUpVph4WRmzW0zWJO2znZKdVujZ0N0S22u",
    ))
    .and(method("GET"))
    .respond_with(ResponseTemplate::new(200).set_body_json(&checkout_session))
    .mount(&app.stripe_server)
    .await;

    let timestamp = chrono::Utc::now().timestamp();
    let mut mac = Hmac::<Sha256>::new_from_slice(app.cfg.stripe.secret.as_bytes()).unwrap();
    mac.update(format!("{}.{}", timestamp, data).as_bytes()); //should it be body
    let hash = mac.finalize();
    let signature = hex::encode(hash.into_bytes());

    let stripe_signature_header = format!("t={},v1={}", timestamp, signature);
    let client = reqwest::Client::new();
    //let body = HashMap::from([("s", "d"), ("t", "v")]);
    let response = client
        .post(&format!("{}/api/v1/payments/callback/stripe", app.address))
        .header("Stripe-Signature", stripe_signature_header)
        .body(data)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(
        200,
        response.status().as_u16(),
        "the api call was not successful"
    );
    let mut conn = app.db.acquire().await.unwrap();
    let p = app
        .repo
        .get_payment_by_id(&app.db, payment.id)
        .await
        .unwrap();
    assert_eq!(p.status, PaymentStatus::Created);
    assert_eq!(p.metadata, None);

    let o = app.repo.get_order_by_id(&mut conn, order.id).await.unwrap();
    assert_eq!(o.status, OrderStatus::Created);

    let user_plan = app.repo.get_user_plan_by_user_id(&app.db, user.id).await;
    match user_plan {
        Ok(_) => panic!("must not be ok"),
        Err(sqlx::Error::RowNotFound) => (),
        Err(err) => panic!("error must be sqlx::Error::RowNotFound,but it was {}", err),
    }
}

#[tokio::test]
async fn stripe_webhook_expired() {
    let app = spawn_app().await;

    let mut conn = app.db.acquire().await.unwrap();

    let (_, user) = app.get_jwt_token_and_user("test@test.test").await;

    let plan = app
        .repo
        .get_plan_by_code(&app.db, PLAN_CODE_1_MONTH)
        .await
        .unwrap();

    let order = app
        .repo
        .create_order(
            &mut conn,
            CreateOrderArgs {
                user_id: user.id,
                plan_id: plan.id,
                total: plan.price,
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
                amount: order.total,
                status: PaymentStatus::Created,
            },
        )
        .await
        .unwrap();

    app.repo
        .update_payment_external_id(
            &mut conn,
            payment.id,
            "cs_test_a11YYufWQzNY63zpQ6QSNRQhkUpVph4WRmzW0zWJO2znZKdVujZ0N0S22u",
        )
        .await
        .unwrap();

    let checkout_session = CheckoutSession {
        id: CheckoutSessionId::from_str(
            "cs_test_a11YYufWQzNY63zpQ6QSNRQhkUpVph4WRmzW0zWJO2znZKdVujZ0N0S22u",
        )
        .unwrap(),
        amount_total: Some(payment.amount.to_i64().unwrap()),
        status: Some(CheckoutSessionStatus::Expired),
        client_reference_id: Some(payment.id.to_string()),
        ..Default::default()
    };

    let e_object = EventObject::CheckoutSession(checkout_session.clone());
    let mut notif_event_data = NotificationEventData {
        ..Default::default()
    };
    notif_event_data.object = e_object; //TODO: why this works, but we cannot set it in the struct
                                        //itself
    let event = Event {
        id: EventId::from_str("evt_e").unwrap(),
        type_: EventType::CheckoutSessionExpired,
        data: notif_event_data,
        ..Default::default()
    };
    let data = serde_json::to_string(&event).unwrap();

    Mock::given(path(
        "/v1/checkout/sessions/cs_test_a11YYufWQzNY63zpQ6QSNRQhkUpVph4WRmzW0zWJO2znZKdVujZ0N0S22u",
    ))
    .and(method("GET"))
    .respond_with(ResponseTemplate::new(200).set_body_json(checkout_session))
    .mount(&app.stripe_server)
    .await;

    let timestamp = chrono::Utc::now().timestamp();
    let mut mac = Hmac::<Sha256>::new_from_slice(app.cfg.stripe.secret.as_bytes()).unwrap();
    mac.update(format!("{}.{}", timestamp, data).as_bytes()); //should it be body
    let hash = mac.finalize();
    let signature = hex::encode(hash.into_bytes());

    let stripe_signature_header = format!("t={},v1={}", timestamp, signature);
    let client = reqwest::Client::new();
    //let body = HashMap::from([("s", "d"), ("t", "v")]);
    let response = client
        .post(&format!("{}/api/v1/payments/callback/stripe", app.address))
        .header("Stripe-Signature", stripe_signature_header)
        .body(data)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(
        200,
        response.status().as_u16(),
        "the api call was not successful"
    );
    let mut conn = app.db.acquire().await.unwrap();
    let p = app
        .repo
        .get_payment_by_id(&app.db, payment.id)
        .await
        .unwrap();
    assert_eq!(p.status, PaymentStatus::Failed);

    let o = app.repo.get_order_by_id(&mut conn, order.id).await.unwrap();
    assert_eq!(o.status, OrderStatus::Failed);

    let user_plan = app.repo.get_user_plan_by_user_id(&app.db, user.id).await;
    match user_plan {
        Ok(_) => panic!("must not be ok"),
        Err(sqlx::Error::RowNotFound) => (),
        Err(err) => panic!("error must be sqlx::Error::RowNotFound,but it was {}", err),
    }
}

#[tokio::test]
async fn stripe_webhook_completed_first_time() {
    let app = spawn_app().await;

    let mut conn = app.db.acquire().await.unwrap();

    let (_, user) = app.get_jwt_token_and_user("test@test.test").await;

    let plan = app
        .repo
        .get_plan_by_code(&app.db, PLAN_CODE_1_MONTH)
        .await
        .unwrap();

    let order = app
        .repo
        .create_order(
            &mut conn,
            CreateOrderArgs {
                user_id: user.id,
                plan_id: plan.id,
                total: plan.price,
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
                amount: order.total,
                status: PaymentStatus::Created,
            },
        )
        .await
        .unwrap();

    app.repo
        .update_payment_external_id(
            &mut conn,
            payment.id,
            "cs_test_a11YYufWQzNY63zpQ6QSNRQhkUpVph4WRmzW0zWJO2znZKdVujZ0N0S22u",
        )
        .await
        .unwrap();

    let checkout_session = CheckoutSession {
        id: CheckoutSessionId::from_str(
            "cs_test_a11YYufWQzNY63zpQ6QSNRQhkUpVph4WRmzW0zWJO2znZKdVujZ0N0S22u",
        )
        .unwrap(),
        amount_total: Some(payment.amount.to_i64().unwrap()),
        status: Some(CheckoutSessionStatus::Complete),
        client_reference_id: Some(payment.id.to_string()),
        ..Default::default()
    };

    let e_object = EventObject::CheckoutSession(checkout_session.clone());
    let mut notif_event_data = NotificationEventData {
        ..Default::default()
    };
    notif_event_data.object = e_object; //TODO: why this works, but we cannot set it in the struct
                                        //itself
    let event = Event {
        id: EventId::from_str("evt_e").unwrap(),
        type_: EventType::CheckoutSessionCompleted,
        data: notif_event_data,
        ..Default::default()
    };
    let data = serde_json::to_string(&event).unwrap();

    Mock::given(path(
        "/v1/checkout/sessions/cs_test_a11YYufWQzNY63zpQ6QSNRQhkUpVph4WRmzW0zWJO2znZKdVujZ0N0S22u",
    ))
    .and(method("GET"))
    .respond_with(ResponseTemplate::new(200).set_body_json(&checkout_session))
    .mount(&app.stripe_server)
    .await;

    let timestamp = chrono::Utc::now().timestamp();
    let mut mac = Hmac::<Sha256>::new_from_slice(app.cfg.stripe.secret.as_bytes()).unwrap();
    mac.update(format!("{}.{}", timestamp, data).as_bytes()); //should it be body
    let hash = mac.finalize();
    let signature = hex::encode(hash.into_bytes());

    let stripe_signature_header = format!("t={},v1={}", timestamp, signature);
    let client = reqwest::Client::new();
    //let body = HashMap::from([("s", "d"), ("t", "v")]);
    let response = client
        .post(&format!("{}/api/v1/payments/callback/stripe", app.address))
        .header("Stripe-Signature", stripe_signature_header)
        .body(data)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(
        200,
        response.status().as_u16(),
        "the api call was not successful"
    );
    let mut conn = app.db.acquire().await.unwrap();
    let p = app
        .repo
        .get_payment_by_id(&app.db, payment.id)
        .await
        .unwrap();
    assert_eq!(p.status, PaymentStatus::Completed);
    let metadata = serde_json::to_string(&checkout_session).unwrap();

    assert_eq!(p.metadata.unwrap(), metadata);

    let o = app.repo.get_order_by_id(&mut conn, order.id).await.unwrap();
    assert_eq!(o.status, OrderStatus::Completed);

    let user_plan = app
        .repo
        .get_user_plan_by_user_id(&app.db, user.id)
        .await
        .unwrap();
    assert_eq!(user_plan.user_id, user.id);
    assert_eq!(user_plan.last_plan_id, plan.id);
    assert_eq!(user_plan.last_order_id, o.id);
    assert_gt!((user_plan.expires_at - chrono::Utc::now()).num_days(), 29);
}

#[tokio::test]
async fn stripe_webhook_completed_already_has_non_expired_1_month_user_plan() {
    let app = spawn_app().await;

    let mut conn = app.db.acquire().await.unwrap();

    let (_, user) = app.get_jwt_token_and_user("test@test.test").await;

    let plan = app
        .repo
        .get_plan_by_code(&app.db, PLAN_CODE_1_MONTH)
        .await
        .unwrap();

    let old_order = app
        .repo
        .create_order(
            &mut conn,
            CreateOrderArgs {
                user_id: user.id,
                plan_id: plan.id,
                total: plan.price.clone(),
                status: OrderStatus::Completed,
            },
        )
        .await
        .unwrap();

    let order = app
        .repo
        .create_order(
            &mut conn,
            CreateOrderArgs {
                user_id: user.id,
                plan_id: plan.id,
                total: plan.price,
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
                amount: order.total,
                status: PaymentStatus::Created,
            },
        )
        .await
        .unwrap();

    app.repo
        .update_payment_external_id(
            &mut conn,
            payment.id,
            "cs_test_a11YYufWQzNY63zpQ6QSNRQhkUpVph4WRmzW0zWJO2znZKdVujZ0N0S22u",
        )
        .await
        .unwrap();

    app.repo
        .create_user_plan_or_update_expires_at(
            &mut conn,
            CreateUserPlanOrUpdateExpiresAtArgs {
                user_id: user.id,
                plan_id: plan.id,
                order_id: old_order.id,
                days: 30,
            },
        )
        .await
        .unwrap();

    let checkout_session = CheckoutSession {
        id: CheckoutSessionId::from_str(
            "cs_test_a11YYufWQzNY63zpQ6QSNRQhkUpVph4WRmzW0zWJO2znZKdVujZ0N0S22u",
        )
        .unwrap(),
        amount_total: Some(payment.amount.to_i64().unwrap()),
        status: Some(CheckoutSessionStatus::Complete),
        client_reference_id: Some(payment.id.to_string()),
        ..Default::default()
    };

    let e_object = EventObject::CheckoutSession(checkout_session.clone());
    let mut notif_event_data = NotificationEventData {
        ..Default::default()
    };
    notif_event_data.object = e_object; //TODO: why this works, but we cannot set it in the struct
                                        //itself
    let event = Event {
        id: EventId::from_str("evt_e").unwrap(),
        type_: EventType::CheckoutSessionCompleted,
        data: notif_event_data,
        ..Default::default()
    };
    let data = serde_json::to_string(&event).unwrap();

    Mock::given(path(
        "/v1/checkout/sessions/cs_test_a11YYufWQzNY63zpQ6QSNRQhkUpVph4WRmzW0zWJO2znZKdVujZ0N0S22u",
    ))
    .and(method("GET"))
    .respond_with(ResponseTemplate::new(200).set_body_json(&checkout_session))
    .mount(&app.stripe_server)
    .await;

    let timestamp = chrono::Utc::now().timestamp();
    let mut mac = Hmac::<Sha256>::new_from_slice(app.cfg.stripe.secret.as_bytes()).unwrap();
    mac.update(format!("{}.{}", timestamp, data).as_bytes()); //should it be body
    let hash = mac.finalize();
    let signature = hex::encode(hash.into_bytes());

    let stripe_signature_header = format!("t={},v1={}", timestamp, signature);
    let client = reqwest::Client::new();
    //let body = HashMap::from([("s", "d"), ("t", "v")]);
    let response = client
        .post(&format!("{}/api/v1/payments/callback/stripe", app.address))
        .header("Stripe-Signature", stripe_signature_header)
        .body(data)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(
        200,
        response.status().as_u16(),
        "the api call was not successful"
    );
    let mut conn = app.db.acquire().await.unwrap();
    let p = app
        .repo
        .get_payment_by_id(&app.db, payment.id)
        .await
        .unwrap();
    assert_eq!(p.status, PaymentStatus::Completed);
    let metadata = serde_json::to_string(&checkout_session).unwrap();

    assert_eq!(p.metadata.unwrap(), metadata);

    let o = app.repo.get_order_by_id(&mut conn, order.id).await.unwrap();
    assert_eq!(o.status, OrderStatus::Completed);

    let user_plan = app
        .repo
        .get_user_plan_by_user_id(&app.db, user.id)
        .await
        .unwrap();
    assert_eq!(user_plan.user_id, user.id);
    assert_eq!(user_plan.last_plan_id, plan.id);
    assert_eq!(user_plan.last_order_id, o.id);
    assert_gt!((user_plan.expires_at - chrono::Utc::now()).num_days(), 59);
}

#[tokio::test]
async fn stripe_webhook_completed_already_has_non_expired_1_month_user_plan_buys_3_month() {
    let app = spawn_app().await;

    let mut conn = app.db.acquire().await.unwrap();

    let (_, user) = app.get_jwt_token_and_user("test@test.test").await;

    let plan = app
        .repo
        .get_plan_by_code(&app.db, PLAN_CODE_1_MONTH)
        .await
        .unwrap();

    let plan_3month = app
        .repo
        .get_plan_by_code(&app.db, PLAN_CODE_3_MONTH)
        .await
        .unwrap();

    let old_order = app
        .repo
        .create_order(
            &mut conn,
            CreateOrderArgs {
                user_id: user.id,
                plan_id: plan.id,
                total: plan.price.clone(),
                status: OrderStatus::Completed,
            },
        )
        .await
        .unwrap();

    let order = app
        .repo
        .create_order(
            &mut conn,
            CreateOrderArgs {
                user_id: user.id,
                plan_id: plan_3month.id,
                total: plan_3month.price,
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
                amount: order.total,
                status: PaymentStatus::Created,
            },
        )
        .await
        .unwrap();

    app.repo
        .update_payment_external_id(
            &mut conn,
            payment.id,
            "cs_test_a11YYufWQzNY63zpQ6QSNRQhkUpVph4WRmzW0zWJO2znZKdVujZ0N0S22u",
        )
        .await
        .unwrap();

    app.repo
        .create_user_plan_or_update_expires_at(
            &mut conn,
            CreateUserPlanOrUpdateExpiresAtArgs {
                user_id: user.id,
                plan_id: plan.id,
                order_id: old_order.id,
                days: 30,
            },
        )
        .await
        .unwrap();

    let checkout_session = CheckoutSession {
        id: CheckoutSessionId::from_str(
            "cs_test_a11YYufWQzNY63zpQ6QSNRQhkUpVph4WRmzW0zWJO2znZKdVujZ0N0S22u",
        )
        .unwrap(),
        amount_total: Some(payment.amount.to_i64().unwrap()),
        status: Some(CheckoutSessionStatus::Complete),
        client_reference_id: Some(payment.id.to_string()),
        ..Default::default()
    };

    let e_object = EventObject::CheckoutSession(checkout_session.clone());
    let mut notif_event_data = NotificationEventData {
        ..Default::default()
    };
    notif_event_data.object = e_object; //TODO: why this works, but we cannot set it in the struct
                                        //itself
    let event = Event {
        id: EventId::from_str("evt_e").unwrap(),
        type_: EventType::CheckoutSessionCompleted,
        data: notif_event_data,
        ..Default::default()
    };
    let data = serde_json::to_string(&event).unwrap();

    Mock::given(path(
        "/v1/checkout/sessions/cs_test_a11YYufWQzNY63zpQ6QSNRQhkUpVph4WRmzW0zWJO2znZKdVujZ0N0S22u",
    ))
    .and(method("GET"))
    .respond_with(ResponseTemplate::new(200).set_body_json(&checkout_session))
    .mount(&app.stripe_server)
    .await;

    let timestamp = chrono::Utc::now().timestamp();
    let mut mac = Hmac::<Sha256>::new_from_slice(app.cfg.stripe.secret.as_bytes()).unwrap();
    mac.update(format!("{}.{}", timestamp, data).as_bytes()); //should it be body
    let hash = mac.finalize();
    let signature = hex::encode(hash.into_bytes());

    let stripe_signature_header = format!("t={},v1={}", timestamp, signature);
    let client = reqwest::Client::new();
    //let body = HashMap::from([("s", "d"), ("t", "v")]);
    let response = client
        .post(&format!("{}/api/v1/payments/callback/stripe", app.address))
        .header("Stripe-Signature", stripe_signature_header)
        .body(data)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(
        200,
        response.status().as_u16(),
        "the api call was not successful"
    );
    let mut conn = app.db.acquire().await.unwrap();
    let p = app
        .repo
        .get_payment_by_id(&app.db, payment.id)
        .await
        .unwrap();
    assert_eq!(p.status, PaymentStatus::Completed);
    let metadata = serde_json::to_string(&checkout_session).unwrap();

    assert_eq!(p.metadata.unwrap(), metadata);

    let o = app.repo.get_order_by_id(&mut conn, order.id).await.unwrap();
    assert_eq!(o.status, OrderStatus::Completed);

    let user_plan = app
        .repo
        .get_user_plan_by_user_id(&app.db, user.id)
        .await
        .unwrap();
    assert_eq!(user_plan.user_id, user.id);
    assert_eq!(user_plan.last_plan_id, plan_3month.id);
    assert_eq!(user_plan.last_order_id, o.id);

    assert_gt!((user_plan.expires_at - chrono::Utc::now()).num_days(), 119);
}

#[tokio::test]
async fn stripe_webhook_completed_already_has_old_expired_1_month_user_plan_buys_3_month() {
    let app = spawn_app().await;

    let mut conn = app.db.acquire().await.unwrap();

    let (_, user) = app.get_jwt_token_and_user("test@test.test").await;

    let plan = app
        .repo
        .get_plan_by_code(&app.db, PLAN_CODE_1_MONTH)
        .await
        .unwrap();

    let plan_3month = app
        .repo
        .get_plan_by_code(&app.db, PLAN_CODE_3_MONTH)
        .await
        .unwrap();

    let old_order = app
        .repo
        .create_order(
            &mut conn,
            CreateOrderArgs {
                user_id: user.id,
                plan_id: plan.id,
                total: plan.price.clone(),
                status: OrderStatus::Completed,
            },
        )
        .await
        .unwrap();

    let order = app
        .repo
        .create_order(
            &mut conn,
            CreateOrderArgs {
                user_id: user.id,
                plan_id: plan_3month.id,
                total: plan_3month.price,
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
                amount: order.total,
                status: PaymentStatus::Created,
            },
        )
        .await
        .unwrap();

    app.repo
        .update_payment_external_id(
            &mut conn,
            payment.id,
            "cs_test_a11YYufWQzNY63zpQ6QSNRQhkUpVph4WRmzW0zWJO2znZKdVujZ0N0S22u",
        )
        .await
        .unwrap();

    app.repo
        .create_user_plan_or_update_expires_at(
            &mut conn,
            CreateUserPlanOrUpdateExpiresAtArgs {
                user_id: user.id,
                plan_id: plan.id,
                order_id: old_order.id,
                days: -30,
            },
        )
        .await
        .unwrap();

    let checkout_session = CheckoutSession {
        id: CheckoutSessionId::from_str(
            "cs_test_a11YYufWQzNY63zpQ6QSNRQhkUpVph4WRmzW0zWJO2znZKdVujZ0N0S22u",
        )
        .unwrap(),
        amount_total: Some(payment.amount.to_i64().unwrap()),
        status: Some(CheckoutSessionStatus::Complete),
        client_reference_id: Some(payment.id.to_string()),
        ..Default::default()
    };

    let e_object = EventObject::CheckoutSession(checkout_session.clone());
    let mut notif_event_data = NotificationEventData {
        ..Default::default()
    };
    notif_event_data.object = e_object; //TODO: why this works, but we cannot set it in the struct
                                        //itself
    let event = Event {
        id: EventId::from_str("evt_e").unwrap(),
        type_: EventType::CheckoutSessionCompleted,
        data: notif_event_data,
        ..Default::default()
    };
    let data = serde_json::to_string(&event).unwrap();

    Mock::given(path(
        "/v1/checkout/sessions/cs_test_a11YYufWQzNY63zpQ6QSNRQhkUpVph4WRmzW0zWJO2znZKdVujZ0N0S22u",
    ))
    .and(method("GET"))
    .respond_with(ResponseTemplate::new(200).set_body_json(&checkout_session))
    .mount(&app.stripe_server)
    .await;

    let timestamp = chrono::Utc::now().timestamp();
    let mut mac = Hmac::<Sha256>::new_from_slice(app.cfg.stripe.secret.as_bytes()).unwrap();
    mac.update(format!("{}.{}", timestamp, data).as_bytes()); //should it be body
    let hash = mac.finalize();
    let signature = hex::encode(hash.into_bytes());

    let stripe_signature_header = format!("t={},v1={}", timestamp, signature);
    let client = reqwest::Client::new();
    //let body = HashMap::from([("s", "d"), ("t", "v")]);
    let response = client
        .post(&format!("{}/api/v1/payments/callback/stripe", app.address))
        .header("Stripe-Signature", stripe_signature_header)
        .body(data)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(
        200,
        response.status().as_u16(),
        "the api call was not successful"
    );
    let mut conn = app.db.acquire().await.unwrap();
    let p = app
        .repo
        .get_payment_by_id(&app.db, payment.id)
        .await
        .unwrap();
    assert_eq!(p.status, PaymentStatus::Completed);
    let metadata = serde_json::to_string(&checkout_session).unwrap();

    assert_eq!(p.metadata.unwrap(), metadata);

    let o = app.repo.get_order_by_id(&mut conn, order.id).await.unwrap();
    assert_eq!(o.status, OrderStatus::Completed);

    let user_plan = app
        .repo
        .get_user_plan_by_user_id(&app.db, user.id)
        .await
        .unwrap();
    assert_eq!(user_plan.user_id, user.id);
    assert_eq!(user_plan.last_plan_id, plan_3month.id);
    assert_eq!(user_plan.last_order_id, o.id);

    assert_gt!((user_plan.expires_at - chrono::Utc::now()).num_days(), 89);
}
