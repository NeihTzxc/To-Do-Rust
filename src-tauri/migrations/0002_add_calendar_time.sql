-- Migration 0002: Add calendar time-blocking columns (start_time, end_time)
ALTER TABLE tasks ADD COLUMN start_time INTEGER;
ALTER TABLE tasks ADD COLUMN end_time INTEGER;

CREATE INDEX IF NOT EXISTS idx_tasks_start_time ON tasks(start_time);
CREATE INDEX IF NOT EXISTS idx_tasks_end_time ON tasks(end_time);

