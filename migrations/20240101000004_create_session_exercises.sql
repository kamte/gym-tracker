CREATE TABLE IF NOT EXISTS session_exercises (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workout_session_id INTEGER NOT NULL REFERENCES workout_sessions(id) ON DELETE CASCADE,
    exercise_id INTEGER NOT NULL REFERENCES exercises(id),
    planned_sets INTEGER NOT NULL DEFAULT 3,
    planned_reps INTEGER NOT NULL DEFAULT 10,
    planned_weight_kg REAL NOT NULL DEFAULT 0.0,
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_session_exercises_session ON session_exercises(workout_session_id);
