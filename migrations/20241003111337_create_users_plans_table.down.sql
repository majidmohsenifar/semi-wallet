-- Add down migration script here

DROP INDEX IF EXISTS idx_users_plans_expires_at;
DROP TABLE users_plans;
