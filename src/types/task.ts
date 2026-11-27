export type TaskStatus = 'todo' | 'in_progress' | 'done';

export type Priority = 1 | 2 | 3;

export interface Task {
  id: string;
  user_id?: string | null;
  title: string;
  description: string | null;
  status: TaskStatus;
  priority: Priority;
  due_date: number | null;
  start_time: number | null;
  end_time: number | null;
  is_all_day: boolean;
  recurrence_rule: string | null;
  created_at: number;
  updated_at: number;
  deleted_at: number | null;
  sync_status: string;
}

export interface UserInfo {
  user_id: string;
  email: string;
}

export interface SyncStatusState {
  is_logged_in: boolean;
  user_email: string | null;
  state: 'idle' | 'syncing' | 'synced' | 'offline' | 'error';
  pending_count: number;
  last_synced_at: number | null;
  error_message: string | null;
}

export type FilterType = 'all' | 'today' | 'high_priority' | 'completed';

export type ViewMode = 'list' | 'kanban';

export type CalendarViewMode = 'day' | 'week' | 'month' | 'agenda';

export type NavigationTab = 'tasks' | 'calendar' | 'search';

export interface CreateTaskPayload {
  title: string;
  description?: string;
  priority?: number;
  due_date?: number | null;
  start_time?: number | null;
  end_time?: number | null;
  is_all_day?: boolean;
  recurrence_rule?: string | null;
}

export interface UpdateTaskPayload {
  id: string;
  title: string;
  description?: string | null;
  status: TaskStatus;
  priority: Priority;
  due_date?: number | null;
  start_time?: number | null;
  end_time?: number | null;
  is_all_day?: boolean;
  recurrence_rule?: string | null;
}
