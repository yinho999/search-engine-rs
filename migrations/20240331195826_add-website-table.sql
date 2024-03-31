-- Add migration script here
CREATE TABLE websites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    url VARCHAR(255) UNIQUE NOT NULL,
    word_count INT NOT NULL DEFAULT 0
);

