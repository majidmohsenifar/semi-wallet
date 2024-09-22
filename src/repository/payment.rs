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
        //conn: A,
        conn: &mut PgConnection,
        args: CreatePaymentArgs,
    ) -> Result<Payment, sqlx::Error>
where {
        let res = sqlx::query_as::<_, Payment>(
            "INSERT INTO payments (
            user_id,
            status,
            amount,
            order_id,
            created_at,
            updated_at
            ) VALUS(
            $1, $2, $3, $4, now(), now()
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
}
