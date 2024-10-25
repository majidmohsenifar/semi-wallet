use chrono;
use serde::{Deserialize, Serialize};
use sqlx::types::{BigDecimal, JsonValue};

#[derive(PartialEq, Serialize, Deserialize, Debug, sqlx::Type)]
#[sqlx(type_name = "order_status", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderStatus {
    Created,
    Completed,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "payment_status", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum PaymentStatus {
    Created,
    Completed,
    Failed,
}

#[derive(Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct Order {
    pub id: i64,
    pub user_id: i64,
    pub plan_id: i64,
    pub total: BigDecimal,
    pub status: OrderStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct Plan {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub price: BigDecimal,
    pub duration: i16,
    pub save_percentage: i16,
}

#[derive(Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct Payment {
    pub id: i64,
    pub user_id: i64,
    pub status: PaymentStatus,
    pub amount: BigDecimal,
    pub order_id: i64,
    pub external_id: Option<String>,
    pub payment_provider_code: String,
    pub payment_url: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: Option<JsonValue>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Debug)]
pub struct Coin {
    pub id: i64,
    pub symbol: String,
    pub name: String,
    pub logo: String,
    pub network: String,
    pub decimals: i8,
    pub contract_address: Option<String>,
    pub description: Option<String>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Debug, Clone, Default)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct UserPlan {
    pub id: i64,
    pub user_id: i64,
    pub last_plan_id: i64,
    pub last_order_id: i64,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct UserCoin {
    pub id: i64,
    pub user_id: i64,
    pub coin_id: i64,
    pub address: String,
    pub symbol: String,
    pub network: String,
    pub amount: Option<BigDecimal>,
    pub amount_updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
