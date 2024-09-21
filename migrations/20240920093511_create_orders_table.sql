-- Add migration script here
CREATE TYPE order_status AS ENUM (
    'CREATED',  
    'COMPLETED',
    'FAILED'
);

CREATE TABLE IF NOT EXISTS orders (
    id SERIAL PRIMARY KEY,
    user_id bigint NOT NULL REFERENCES users(id),
    plan_id bigint NOT NULL REFERENCES plans(id),
    total DECIMAL NOT NULL,
    status order_status NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

