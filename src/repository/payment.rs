use sqlx::PgConnection;

use super::{
    db::Repository,
    models::{Payment, PaymentStatus},
};

pub struct CreatePaymentArgs {
    pub user_id: i64,
    pub order_id: i64,
    pub amount: f64,
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
            created_at,
            updated_at
            ) VALUS(
            $1, $2, $3, $4, NOW(), NOW()
            ) RETURNING *",
        )
        .bind(args.user_id)
        .bind(args.amount)
        .bind(args.order_id)
        .bind(args.status)
        .fetch_one(&mut *conn)
        .await?;
        Ok(res)
    }

    pub async fn update_payment_external_id(
        &self,
        conn: &mut PgConnection,
        payment_id: i64,
        external_id: String,
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
}
