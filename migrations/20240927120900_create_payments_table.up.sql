-- Add up migration script here
CREATE TYPE payment_status AS ENUM (
    'CREATED',  
    'COMPLETED',
    'FAILED'
);

CREATE TABLE IF NOT EXISTS payments (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    status payment_status NOT NULL,
    amount NUMERIC NOT NULL,
    order_id BIGINT NOT NULL REFERENCES orders(id),
    external_id VARCHAR(128),
    payment_provider_code VARCHAR(48) NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
