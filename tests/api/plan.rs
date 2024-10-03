use semi_wallet::handler::response::ApiResponse;
use semi_wallet::service::plan::service::Plan;

use crate::helpers::spawn_app;

#[tokio::test]
async fn get_plans_list_successful() {
    let app = spawn_app().await;
    //we already have plans in our db as we done it in one of our migrations
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/api/v1/plans", app.address))
        .send()
        .await
        .expect("failed to execute request");
    assert_eq!(200, response.status().as_u16(),);
    let bytes = response.bytes().await.unwrap();
    let res: ApiResponse<'_, Vec<Plan>> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(res.message, "");
    //TODO: validate the list itself
    let data = res.data.unwrap();
    assert_eq!(data.len(), 4);
    let plan1 = data.first().unwrap();
    assert_ne!(plan1.id, 0);
    assert_eq!(plan1.code, "1_MONTH");
    assert_eq!(plan1.name, "One Month");
    assert_eq!(plan1.price, 2.0);
    assert_eq!(plan1.duration, 1);
    assert_eq!(plan1.save_percentage, 0);

    let plan2 = data.get(1).unwrap();
    assert_ne!(plan2.id, 0);
    assert_eq!(plan2.code, "3_MONTH");
    assert_eq!(plan2.name, "3 Months");
    assert_eq!(plan2.price, 5.7);
    assert_eq!(plan2.duration, 3);
    assert_eq!(plan2.save_percentage, 5);

    let plan3 = data.get(2).unwrap();
    assert_ne!(plan3.id, 0);
    assert_eq!(plan3.code, "6_MONTH");
    assert_eq!(plan3.name, "6 Months");
    assert_eq!(plan3.price, 9.60);
    assert_eq!(plan3.duration, 6);
    assert_eq!(plan3.save_percentage, 10);

    let plan4 = data.get(3).unwrap();
    assert_ne!(plan4.id, 0);
    assert_eq!(plan4.code, "12_MONTH");
    assert_eq!(plan4.name, "12 Months");
    assert_eq!(plan4.price, 19.20);
    assert_eq!(plan4.duration, 12);
    assert_eq!(plan4.save_percentage, 20);
}
