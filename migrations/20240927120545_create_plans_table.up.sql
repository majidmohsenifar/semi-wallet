-- Add up migration script here
CREATE TABLE IF NOT EXISTS plans (
    id bigserial PRIMARY KEY,
    code VARCHAR(20) UNIQUE NOT NULL,
    name VARCHAR(100) UNIQUE NOT NULL,
    price NUMERIC(5, 2) NOT NULL,
    duration SMALLINT NOT NULL,
    save_percentage SMALLINT NOT NULL
);
