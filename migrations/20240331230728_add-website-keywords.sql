-- Add migration script here

CREATE TABLE website_keywords (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    keyword UUID NOT NULL,
    website_id UUID NOT NULL,
    FOREIGN KEY (website_id) REFERENCES websites(id),
    FOREIGN KEY (keyword) REFERENCES keywords(id)
);
