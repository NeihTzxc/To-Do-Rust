-- Migration 0004: Thêm is_all_day và recurrence_rule cho All-day, Date Range & Recurring Tasks
ALTER TABLE tasks ADD COLUMN is_all_day INTEGER NOT NULL DEFAULT 0;
ALTER TABLE tasks ADD COLUMN recurrence_rule TEXT;
