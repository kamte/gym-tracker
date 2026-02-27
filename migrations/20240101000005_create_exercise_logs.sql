CREATE TABLE IF NOT EXISTS exercise_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id),
    exercise_id INTEGER NOT NULL REFERENCES exercises(id),
    workout_session_id INTEGER REFERENCES workout_sessions(id),
    set_number INTEGER NOT NULL DEFAULT 1,
    reps_completed INTEGER NOT NULL,
    weight_kg REAL NOT NULL DEFAULT 0.0,
    rpe REAL,
    performed_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_exercise_logs_user_id ON exercise_logs(user_id);
CREATE INDEX idx_exercise_logs_exercise_id ON exercise_logs(exercise_id);
