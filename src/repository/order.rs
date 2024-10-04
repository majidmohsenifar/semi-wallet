use sqlx::types::BigDecimal;
use sqlx::PgConnection;

use super::{
    db::Repository,
    models::{Order, OrderStatus},
};

pub struct CreateOrderArgs {
    pub user_id: i64,
    pub plan_id: i64,
    pub total: BigDecimal,
    pub status: OrderStatus,
}

impl Repository {
    pub async fn create_order(
        &self,
        conn: &mut PgConnection,
        args: CreateOrderArgs,
    ) -> Result<Order, sqlx::Error> {
        let res = sqlx::query_as::<_, Order>(
            "INSERT INTO orders (
            user_id,
            plan_id,
            total,
            status,
            created_at,
            updated_at
            ) VALUES (
            $1, $2, $3, $4, NOW(), NOW()
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

    pub async fn get_order_by_id(
        &self,
        conn: &mut PgConnection,
        id: i64,
    ) -> Result<Order, sqlx::Error> {
        let res = sqlx::query_as::<_, Order>("SELECT * from orders where id = $1")
            .bind(id)
            .fetch_one(&mut *conn)
            .await?;
        Ok(res)
    }
}
