-- Add migration script here
ALTER table subscriptions ADD COLUMN status TEXT NULL;
