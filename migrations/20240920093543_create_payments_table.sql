-- Add migration script here
CREATE TYPE payment_status AS ENUM (
    'CREATED',  
    'IN_PROGRESS'
    'COMPLETED',
    'FAILED'
);

CREATE TABLE IF NOT EXISTS payments (
    id bigserial PRIMARY KEY,
    user_id bigint NOT NULL,
    status payment_status NOT NULL,
    amount numeric NOT NULL,
    order_id bigint NOT NULL REFERENCES orders(id),
    created_at timestamp  NOT NULL DEFAULT now(),
    updated_at timestamp  NOT NULL DEFAULT now()
);

