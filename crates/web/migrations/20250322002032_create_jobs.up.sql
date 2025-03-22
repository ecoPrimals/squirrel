-- Add up migration script here

-- Create jobs table
CREATE TABLE jobs (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    repository_url TEXT NOT NULL,
    git_ref TEXT NOT NULL,
    config TEXT NOT NULL,
    status TEXT NOT NULL,
    progress REAL NOT NULL,
    error TEXT,
    result_url TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
