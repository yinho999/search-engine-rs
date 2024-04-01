-- Add migration script here

CREATE TABLE website_keywords (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    keyword_id UUID NOT NULL,
    website_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (website_id) REFERENCES websites(id),
    FOREIGN KEY (keyword_id) REFERENCES keywords(id)
);
