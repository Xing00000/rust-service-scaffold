-- Add migration script here
CREATE TABLE IF NOT EXISTS users (
  id   UUID PRIMARY KEY,
  name TEXT NOT NULL
);