export type TaskStatus = 'todo' | 'in_progress' | 'done';

export type Priority = 1 | 2 | 3;

export interface Task {
  id: string;
  title: string;
  description: string | null;
  status: TaskStatus;
  priority: Priority;
  due_date: number | null;
  created_at: number;
  updated_at: number;
  deleted_at: number | null;
  sync_status: string;
}

export type FilterType = 'all' | 'today' | 'high_priority' | 'completed';

export type ViewMode = 'list' | 'kanban';

export interface CreateTaskPayload {
  title: string;
  description?: string;
  priority?: number;
  due_date?: number | null;
}

export interface UpdateTaskPayload {
  id: string;
  title: string;
  description?: string | null;
  status: TaskStatus;
  priority: Priority;
  due_date?: number | null;
}
