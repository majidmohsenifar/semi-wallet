-- Add up migration script here
CREATE TABLE IF NOT EXISTS users_plans (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) UNIQUE,
    last_plan_id BIGINT NOT NULL REFERENCES plans(id),
    last_order_id BIGINT NOT NULL REFERENCES orders(id) UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL 
);

CREATE INDEX IF NOT EXISTS idx_users_plans_expires_at ON users_plans(expires_at);

