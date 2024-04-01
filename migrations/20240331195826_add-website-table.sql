-- Add migration script here
-- Set timezone to UTC
SET TIMEZONE='UTC';
CREATE TABLE websites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    url VARCHAR(255) UNIQUE NOT NULL,
    word_count INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

