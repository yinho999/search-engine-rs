-- Add migration script here

CREATE TABLE IF NOT EXISTS public.website_keyword_tfidf
(
    id UUID DEFAULT gen_random_uuid() NOT NULL PRIMARY KEY,
    website_id UUID NOT NULL REFERENCES websites(id),
    keyword_id UUID NOT NULL REFERENCES keywords(id),
    tf numeric NOT NULL,
    idf numeric NOT NULL,
    tfidf numeric NOT NULL,
    created_at timestamptz DEFAULT NOW() NOT NULL,
    updated_at timestamptz DEFAULT NOW() NOT NULL
);