-- We wrap the whole migration in a transaction to make sure it 
-- succeeds or fails automatically. `sqlx` doesn't do this 
-- automatically for us. SQLite doesn't handle nested transactions
-- like BEGIN / COMMIT so we omit them and add sqlx-disable-tx 
-- commented. SQLite also doesn't handle updating the table so we
-- need to create clone, drop and swap

-- sqlx-disable-tx

-- Backfill `status` for historical entries
UPDATE subscriptions
    SET status = 'confirmed'
    WHERE status IS NULL;

-- Create new table with corrected columns, replaces:
-- ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
CREATE TABLE subscriptions_status_fix (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    -- UTC, use chrono to configure timezone
    subscribed_at TEXT DEFAULT (datetime('now', 'utc')),
    status TEXT NOT NULL
);

-- Copy data in
INSERT INTO subscriptions_status_fix (id, name, email, subscribed_at, status)
    SELECT id, name, email, subscribed_at, status
    FROM subscriptions;

DROP TABLE subscriptions;

ALTER TABLE subscriptions_status_fix RENAME TO subscriptions;
