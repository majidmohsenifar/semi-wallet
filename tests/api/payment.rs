use semi_wallet::{
    handler::response::ApiResponse,
    service::payment::service::{
        PaymentProvider, PAYMENT_PROVIDER_BITPAY, PAYMENT_PROVIDER_STRIPE,
    },
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn get_payment_providers_list_successful() {
    let app = spawn_app().await;
    //we already have plans in our db as we done it in one of our migrations
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/api/v1/payments/providers", app.address))
        .send()
        .await
        .expect("failed to execute request");
    assert_eq!(200, response.status().as_u16(),);
    let bytes = response.bytes().await.unwrap();
    let res: ApiResponse<'_, Vec<PaymentProvider>> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(res.message, "");
    let data = res.data.unwrap();
    assert_eq!(data.len(), 2);
    let pp1 = data.first().unwrap();
    assert_eq!(pp1.code, PAYMENT_PROVIDER_STRIPE);
    assert!(pp1.enabled);

    let pp2 = data.get(1).unwrap();
    assert_eq!(pp2.code, PAYMENT_PROVIDER_BITPAY);
    assert!(pp2.enabled);
}
