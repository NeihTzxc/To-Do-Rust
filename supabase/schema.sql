-- =========================================================================
-- Schema & Row Level Security (RLS) cho Supabase PostgreSQL
-- Chạy đoạn script này trên Supabase SQL Editor
-- =========================================================================

-- 1. Tạo bảng tasks cho từng người dùng
CREATE TABLE IF NOT EXISTS public.tasks (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'todo',
    priority INT NOT NULL DEFAULT 2,
    due_date BIGINT,
    start_time BIGINT,
    end_time BIGINT,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    deleted_at BIGINT
);

-- 2. Chỉ mục tối ưu hóa hiệu năng truy vấn và sync
CREATE INDEX IF NOT EXISTS idx_tasks_user_id ON public.tasks(user_id);
CREATE INDEX IF NOT EXISTS idx_tasks_updated_at ON public.tasks(updated_at);
CREATE INDEX IF NOT EXISTS idx_tasks_deleted_at ON public.tasks(deleted_at);

-- 3. Bật Row Level Security (RLS) để cô lập dữ liệu tuyệt đối giữa các user
ALTER TABLE public.tasks ENABLE ROW LEVEL SECURITY;

-- 4. Chính sách bảo mật RLS: Chỉ user đã xác thực (authenticated) được thao tác trên task của chính họ
DROP POLICY IF EXISTS "Users can select own tasks" ON public.tasks;
CREATE POLICY "Users can select own tasks"
    ON public.tasks FOR SELECT
    TO authenticated
    USING (auth.uid() = user_id);

DROP POLICY IF EXISTS "Users can insert own tasks" ON public.tasks;
CREATE POLICY "Users can insert own tasks"
    ON public.tasks FOR INSERT
    TO authenticated
    WITH CHECK (auth.uid() = user_id);

DROP POLICY IF EXISTS "Users can update own tasks" ON public.tasks;
CREATE POLICY "Users can update own tasks"
    ON public.tasks FOR UPDATE
    TO authenticated
    USING (auth.uid() = user_id)
    WITH CHECK (auth.uid() = user_id);

DROP POLICY IF EXISTS "Users can delete own tasks" ON public.tasks;
CREATE POLICY "Users can delete own tasks"
    ON public.tasks FOR DELETE
    TO authenticated
    USING (auth.uid() = user_id);

-- 5. Cấp quyền truy cập schema và bảng cho role authenticated
GRANT USAGE ON SCHEMA public TO authenticated;
GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE public.tasks TO authenticated;

-- 6. Trigger tự động cập nhật updated_at (milliseconds) khi chỉnh sửa task trực tiếp trên Supabase Dashboard
CREATE OR REPLACE FUNCTION public.handle_tasks_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = (EXTRACT(EPOCH FROM clock_timestamp()) * 1000)::BIGINT;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS set_tasks_updated_at ON public.tasks;
CREATE TRIGGER set_tasks_updated_at
    BEFORE UPDATE ON public.tasks
    FOR EACH ROW
    EXECUTE FUNCTION public.handle_tasks_updated_at();


