-- Add up migration script here
CREATE TABLE IF NOT EXISTS coins (
    id BIGSERIAL PRIMARY KEY,
    symbol VARCHAR(8) NOT NULL,
    name VARCHAR(32) NOT NULL,
    logo VARCHAR(252) NOT NULL,
    network VARCHAR(8) NOT NULL,
    price_pair_symbol VARCHAR(12) NULL,
    decimals SMALLINT NOT NULL,
    contract_address VARCHAR(64) NULL,
    description TEXT NULL,
    UNIQUE(symbol, network)
);
