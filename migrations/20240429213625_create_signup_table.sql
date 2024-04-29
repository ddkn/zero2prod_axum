-- Add migration script here
CREATE TABLE signup (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    subscribed_at TEXT default (datetime('now', 'utc')) -- UTC, use chrono to configure timezone
);
