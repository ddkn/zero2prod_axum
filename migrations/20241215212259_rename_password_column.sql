CREATE TABLE users_password_fix(
    user_id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL
);

-- Copy data in
INSERT INTO users_password_fix (user_id, username, password_hash)
    SELECT user_id, username, password
    FROM users;

DROP TABLE users;

ALTER TABLE users_password_fix RENAME TO users;
