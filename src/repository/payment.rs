use sqlx::types::BigDecimal;
use sqlx::{PgConnection, Pool, Postgres};

use super::{
    db::Repository,
    models::{Payment, PaymentStatus},
};

pub struct CreatePaymentArgs {
    pub user_id: i64,
    pub order_id: i64,
    pub payment_provider_code: String,
    pub amount: BigDecimal,
    pub status: PaymentStatus,
}

impl Repository {
    pub async fn create_payment(
        &self,
        conn: &mut PgConnection,
        args: CreatePaymentArgs,
    ) -> Result<Payment, sqlx::Error> {
        let res = sqlx::query_as::<_, Payment>(
            "INSERT INTO payments (
            user_id,
            status,
            amount,
            order_id,
            payment_provider_code,
            created_at,
            updated_at
            ) VALUES(
            $1, $2, $3, $4, $5, NOW(), NOW()
            ) RETURNING *",
        )
        .bind(args.user_id)
        .bind(args.status)
        .bind(args.amount)
        .bind(args.order_id)
        .bind(args.payment_provider_code)
        .fetch_one(&mut *conn)
        .await?;
        Ok(res)
    }

    pub async fn update_payment_external_id(
        &self,
        conn: &mut PgConnection,
        payment_id: i64,
        external_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE payments
            SET external_id = $2,
             updated_at = NOW()
            WHERE id = $1;",
        )
        .bind(payment_id)
        .bind(external_id)
        .execute(&mut *conn)
        .await?;
        Ok(())
    }
    pub async fn get_payment_by_id(
        &self,
        db: &Pool<Postgres>,
        id: i64,
    ) -> Result<Payment, sqlx::Error> {
        let payment = sqlx::query_as::<_, Payment>(
            "SELECT * FROM payments
            WHERE id = $1 ",
        )
        .bind(id)
        .fetch_one(db)
        .await?;
        Ok(payment)
    }

    pub async fn get_last_payment_by_order_id(
        &self,
        db: &Pool<Postgres>,
        order_id: i64,
    ) -> Result<Payment, sqlx::Error> {
        let payment = sqlx::query_as::<_, Payment>(
            "SELECT * FROM payments
            WHERE order_id = $1 ORDER BY id DESC LIMIT 1",
        )
        .bind(order_id)
        .fetch_one(db)
        .await?;
        Ok(payment)
    }

    pub async fn update_payment_status_metadata(
        &self,
        conn: &mut PgConnection,
        payment_id: i64,
        status: PaymentStatus,
        metadata: Option<sqlx::types::JsonValue>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE payments
            SET status = $2,
                metadata = $3,
             updated_at = NOW()
            WHERE id = $1;",
        )
        .bind(payment_id)
        .bind(status)
        .bind(metadata)
        .execute(&mut *conn)
        .await?;
        Ok(())
    }
}
