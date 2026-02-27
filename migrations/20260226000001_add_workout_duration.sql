-- Add workout duration tracking columns
ALTER TABLE workout_sessions ADD COLUMN started_at TEXT;
ALTER TABLE workout_sessions ADD COLUMN completed_at TEXT;
