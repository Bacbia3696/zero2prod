-- Add migration script here
CREATE TABLE
  subscriptions (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    NAME TEXT NOT NULL,
    subscribed_at timestamptz NOT NULL
  )
;
