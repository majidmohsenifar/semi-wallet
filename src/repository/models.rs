use chrono;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[sqlx(type_name = "order_status", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderStatus {
    Created,
    Completed,
    Failed,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[sqlx(type_name = "payment_status", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum PaymentStatus {
    Created,
    Completed,
    Failed,
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct Order {
    pub id: i64,
    pub user_id: i64,
    pub plan_id: i64,
    pub total: f64,
    pub status: OrderStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct Plan {
    pub id: i64,
    pub code: String,
    pub price: f64,
    pub duration: i16,
    pub save_percentage: i16,
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct Payment {
    pub id: i64,
    pub user_id: i64,
    pub status: PaymentStatus,
    pub amount: f64,
    pub order_id: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct Coin {
    pub id: i64,
    pub symbol: String,
    pub name: String,
    pub logo: String,
    pub network: String,
    pub decimals: i16,
    pub description: String,
}
