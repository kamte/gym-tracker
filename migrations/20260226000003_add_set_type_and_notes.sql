-- Add set type and notes to exercise logs
ALTER TABLE exercise_logs ADD COLUMN set_type TEXT NOT NULL DEFAULT 'working' CHECK(set_type IN ('warmup', 'working', 'drop', 'failure'));
ALTER TABLE exercise_logs ADD COLUMN notes TEXT NOT NULL DEFAULT '';
