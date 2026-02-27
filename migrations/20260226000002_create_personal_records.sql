-- Personal records table
CREATE TABLE IF NOT EXISTS personal_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id),
    exercise_id INTEGER NOT NULL REFERENCES exercises(id),
    record_type TEXT NOT NULL CHECK(record_type IN ('max_weight', 'max_reps', 'max_volume', 'estimated_1rm')),
    value REAL NOT NULL,
    exercise_log_id INTEGER REFERENCES exercise_logs(id),
    achieved_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(user_id, exercise_id, record_type)
);
