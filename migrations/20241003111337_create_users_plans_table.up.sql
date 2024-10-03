-- Add up migration script here
CREATE TABLE IF NOT EXISTS users_plans (
    user_id bigint NOT NULL REFERENCES users(id),
    plan_id bigint NOT NULL REFERENCES plans(id),
    expires_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, plan_id)
)
