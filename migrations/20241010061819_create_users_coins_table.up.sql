-- Add up migration script here
CREATE TABLE IF NOT EXISTS users_coins (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    coin_id BIGINT NOT NULL REFERENCES coins(id),
    address VARCHAR(64) NOT NULL,
    symbol VARCHAR(8) NOT NULL,
    network VARCHAR(8) NOT NULL,
    amount NUMERIC(5, 2) NULL,
    amount_updated_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, coin_id)

)
