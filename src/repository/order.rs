use sqlx::PgConnection;

use super::{
    db::Repository,
    models::{Order, OrderStatus},
};

pub struct CreateOrderArgs {
    pub user_id: i64,
    pub plan_id: i64,
    pub total: f64,
    pub status: OrderStatus,
}

impl Repository {
    pub async fn create_order(
        &self,
        conn: &mut PgConnection,
        //conn: A,
        args: CreateOrderArgs,
    ) -> Result<Order, sqlx::Error>
where {
        //let mut conn = conn.acquire().await?;
        let res = sqlx::query_as::<_, Order>(
            "INSERT INTO orders (
            user_id,
            plan_id,
            total,
            status,
            created_at,
            updated_at
            ) VALUS(
            $1, $2, $3, $4, now(), now()
            ) RETURNING *",
        )
        .bind(args.user_id)
        .bind(args.plan_id)
        .bind(args.total)
        .bind(args.status)
        .fetch_one(&mut *conn)
        .await?;
        Ok(res)
    }
}
