-- Add up migration script here
CREATE TABLE IF NOT EXISTS coins (
    id bigserial PRIMARY KEY,
    symbol varchar(8) NOT NULL,
    name varchar(32) NOT NULL,
    logo varchar(252) NOT NULL,
    network varchar(8) NOT NULL,
    decimals smallint NOT NULL,
    description text NULL,
    UNIQUE(symbol,network)
);
