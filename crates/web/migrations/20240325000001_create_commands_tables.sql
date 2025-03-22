-- Create commands tables
-- Schema for command definitions and command executions

-- Command definition table
CREATE TABLE commands (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    parameter_schema TEXT NOT NULL,  -- JSON schema for command parameters
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

-- Command execution history
CREATE TABLE command_executions (
    id TEXT PRIMARY KEY,
    command_name TEXT NOT NULL,
    user_id TEXT NOT NULL,
    parameters TEXT NOT NULL,  -- JSON parameters
    status TEXT NOT NULL,      -- queued, running, completed, failed, cancelled
    progress REAL DEFAULT 0.0,
    result TEXT,               -- JSON result
    error TEXT,                -- Error information if failed
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
); 