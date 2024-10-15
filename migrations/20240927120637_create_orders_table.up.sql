-- Add up migration script here
CREATE TYPE order_status AS ENUM (
    'CREATED',  
    'COMPLETED',
    'FAILED'
);

CREATE TABLE IF NOT EXISTS orders (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    plan_id BIGINT NOT NULL REFERENCES plans(id),
    total NUMERIC(5, 2) NOT NULL,
    status order_status NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

