-- Migration 0003: Thêm user_id, bảng auth_session và sync_meta cho Local-first SQLite
ALTER TABLE tasks ADD COLUMN user_id TEXT;
CREATE INDEX IF NOT EXISTS idx_tasks_user_id ON tasks(user_id);

-- Bảng lưu trữ phiên đăng nhập hiện tại trên máy local (tối đa 1 dòng với id = 1)
CREATE TABLE IF NOT EXISTS auth_session (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    user_id TEXT NOT NULL,
    email TEXT NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    expires_at INTEGER NOT NULL
);

-- Bảng metadata lưu trữ mốc thời gian đồng bộ (last_pulled_at_<user_id>)
CREATE TABLE IF NOT EXISTS sync_meta (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

