CREATE TABLE users(
    user_id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    PASSWORD TEXT NOT NULL
);
