-- Add migration script here

ALTER TABLE website_keywords
ADD COLUMN frequency integer NOT NULL DEFAULT 1;