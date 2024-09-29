-- Add up migration script here
CREATE TABLE IF NOT EXISTS plans (
    id bigserial PRIMARY KEY,
    code varchar(20) UNIQUE NOT NULL,
    price numeric NOT NULL,
    duration smallint NOT NULL,
    save_percentage smallint NOT NULL
);
