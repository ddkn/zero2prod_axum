CREATE TABLE subscription_tokens_fix (
    subscription_token TEXT NOT NULL,
    subscriber_id TEXT NOT NULL
        REFERENCES subscriptions (id),
    PRIMARY KEY (subscription_token)
);

-- Copy data in
INSERT INTO subscription_tokens_fix (subscription_token, subscriber_id)
    SELECT subscription_token, subscriber_id
    FROM subscription_tokens;

DROP TABLE subscription_tokens;

ALTER TABLE subscription_tokens_fix RENAME TO subscription_tokens;
