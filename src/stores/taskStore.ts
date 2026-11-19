import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import type { Task, TaskStatus, Priority, FilterType, ViewMode, CreateTaskPayload, UpdateTaskPayload } from '@/types/task';

function isTodayOrOverdue(timestamp: number | null): boolean {
  if (!timestamp) return false;
  const endOfToday = new Date();
  endOfToday.setHours(23, 59, 59, 999);
  return timestamp <= endOfToday.getTime();
}

// Fallback in-memory initial data for browser preview when tauri is not injected
const mockTasks: Task[] = [
  {
    id: 'demo-1',
    title: 'Chào mừng bạn đến với TickTask!',
    description: 'Ứng dụng quản lý công việc phong cách TickTick được xây dựng trên Tauri v2, Vue 3, Tailwind CSS và SQLite Local-first.',
    status: 'todo',
    priority: 3,
    due_date: Date.now(),
    created_at: Date.now() - 3600000,
    updated_at: Date.now(),
    deleted_at: null,
    sync_status: 'pending',
  },
  {
    id: 'demo-2',
    title: 'Khám phá chế độ xem Bảng Kanban',
    description: 'Thử kéo và thả thẻ này sang cột "Đang làm" hoặc "Hoàn thành" để xem hiệu ứng chuyển trạng thái mượt mà.',
    status: 'in_progress',
    priority: 2,
    due_date: Date.now() + 86400000,
    created_at: Date.now() - 7200000,
    updated_at: Date.now(),
    deleted_at: null,
    sync_status: 'synced',
  },
  {
    id: 'demo-3',
    title: 'Thử thanh thêm nhanh Quick Add',
    description: 'Gõ tiêu đề công việc, chọn cờ ưu tiên và hạn chót ở thanh nhập phía trên rồi nhấn Enter!',
    status: 'done',
    priority: 1,
    due_date: null,
    created_at: Date.now() - 86400000,
    updated_at: Date.now(),
    deleted_at: null,
    sync_status: 'synced',
  },
];

export const useTaskStore = defineStore('tasks', {
  state: () => ({
    tasks: [] as Task[],
    selectedTaskId: null as string | null,
    currentFilter: 'all' as FilterType,
    currentView: 'list' as ViewMode,
    searchQuery: '',
    isLoading: false,
    errorMessage: null as string | null,
    isDarkMode: true,
    isSidebarCollapsed: false,
    isTauriEnv: typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window,
  }),

  getters: {
    selectedTask: (state): Task | null => {
      if (!state.selectedTaskId) return null;
      return state.tasks.find((t) => t.id === state.selectedTaskId) || null;
    },

    activeTasks: (state): Task[] => {
      return state.tasks.filter((t) => t.status !== 'done');
    },

    todayTasks: (state): Task[] => {
      return state.tasks.filter((t) => t.status !== 'done' && isTodayOrOverdue(t.due_date));
    },

    highPriorityTasks: (state): Task[] => {
      return state.tasks.filter((t) => t.status !== 'done' && t.priority === 3);
    },

    completedTasks: (state): Task[] => {
      return state.tasks.filter((t) => t.status === 'done');
    },

    pendingSyncCount: (state): number => {
      return state.tasks.filter((t) => t.sync_status === 'pending').length;
    },

    counts(): Record<FilterType, number> {
      return {
        all: this.activeTasks.length,
        today: this.todayTasks.length,
        high_priority: this.highPriorityTasks.length,
        completed: this.completedTasks.length,
      };
    },

    filteredTasks(): Task[] {
      let result: Task[] = [];
      switch (this.currentFilter) {
        case 'today':
          result = this.todayTasks;
          break;
        case 'high_priority':
          result = this.highPriorityTasks;
          break;
        case 'completed':
          result = this.completedTasks;
          break;
        case 'all':
        default:
          result = this.activeTasks;
          break;
      }

      if (this.searchQuery.trim()) {
        const query = this.searchQuery.toLowerCase();
        result = result.filter(
          (t) =>
            t.title.toLowerCase().includes(query) ||
            (t.description && t.description.toLowerCase().includes(query))
        );
      }

      return result;
    },

    kanbanColumns(): Record<TaskStatus, Task[]> {
      let baseList = this.tasks;
      if (this.searchQuery.trim()) {
        const query = this.searchQuery.toLowerCase();
        baseList = baseList.filter(
          (t) =>
            t.title.toLowerCase().includes(query) ||
            (t.description && t.description.toLowerCase().includes(query))
        );
      }

      if (this.currentFilter === 'today') {
        baseList = baseList.filter((t) => t.status === 'done' || isTodayOrOverdue(t.due_date));
      } else if (this.currentFilter === 'high_priority') {
        baseList = baseList.filter((t) => t.priority === 3);
      }

      return {
        todo: baseList.filter((t) => t.status === 'todo'),
        in_progress: baseList.filter((t) => t.status === 'in_progress'),
        done: baseList.filter((t) => t.status === 'done'),
      };
    },
  },

  actions: {
    async init() {
      // Load dark mode preference
      const savedTheme = localStorage.getItem('theme');
      if (savedTheme) {
        this.isDarkMode = savedTheme === 'dark';
      } else {
        this.isDarkMode = true; // default Dark mode
      }
      this.applyTheme();

      await this.fetchTasks();
    },

    applyTheme() {
      if (this.isDarkMode) {
        document.documentElement.classList.add('dark');
        localStorage.setItem('theme', 'dark');
      } else {
        document.documentElement.classList.remove('dark');
        localStorage.setItem('theme', 'light');
      }
    },

    toggleTheme() {
      this.isDarkMode = !this.isDarkMode;
      this.applyTheme();
    },

    toggleSidebar() {
      this.isSidebarCollapsed = !this.isSidebarCollapsed;
    },

    setFilter(filter: FilterType) {
      this.currentFilter = filter;
    },

    setView(view: ViewMode) {
      this.currentView = view;
    },

    selectTask(id: string | null) {
      this.selectedTaskId = id;
    },

    async fetchTasks() {
      this.isLoading = true;
      this.errorMessage = null;
      try {
        if (this.isTauriEnv) {
          const list = await invoke<Task[]>('get_tasks');
          this.tasks = list;
        } else {
          // Dev mock fallback
          if (this.tasks.length === 0) {
            this.tasks = [...mockTasks];
          }
        }
      } catch (err: any) {
        console.error('Lỗi tải danh sách task:', err);
        this.errorMessage = String(err);
        if (!this.isTauriEnv && this.tasks.length === 0) {
          this.tasks = [...mockTasks];
        }
      } finally {
        this.isLoading = false;
      }
    },

    async createTask(payload: CreateTaskPayload): Promise<Task | null> {
      try {
        if (this.isTauriEnv) {
          const created = await invoke<Task>('create_task', {
            title: payload.title,
            description: payload.description ?? null,
            priority: payload.priority ?? 2,
            dueDate: payload.due_date ?? null,
          });
          this.tasks.unshift(created);
          return created;
        } else {
          const now = Date.now();
          const newTask: Task = {
            id: 'mock-' + Math.random().toString(36).substring(2, 9),
            title: payload.title,
            description: payload.description ?? null,
            status: 'todo',
            priority: (payload.priority as Priority) ?? 2,
            due_date: payload.due_date ?? null,
            created_at: now,
            updated_at: now,
            deleted_at: null,
            sync_status: 'pending',
          };
          this.tasks.unshift(newTask);
          return newTask;
        }
      } catch (err: any) {
        console.error('Lỗi thêm task:', err);
        this.errorMessage = String(err);
        return null;
      }
    },

    async updateTask(payload: UpdateTaskPayload): Promise<Task | null> {
      const idx = this.tasks.findIndex((t) => t.id === payload.id);
      if (idx !== -1) {
        // Optimistic local update
        this.tasks[idx] = {
          ...this.tasks[idx],
          title: payload.title,
          description: payload.description ?? null,
          status: payload.status,
          priority: payload.priority,
          due_date: payload.due_date ?? null,
          updated_at: Date.now(),
          sync_status: 'pending',
        };
      }

      try {
        if (this.isTauriEnv) {
          const updated = await invoke<Task>('update_task', {
            id: payload.id,
            title: payload.title,
            description: payload.description ?? null,
            status: payload.status,
            priority: payload.priority,
            dueDate: payload.due_date ?? null,
          });
          if (idx !== -1) {
            this.tasks[idx] = updated;
          }
          return updated;
        }
        return this.tasks[idx] ?? null;
      } catch (err: any) {
        console.error('Lỗi cập nhật task:', err);
        this.errorMessage = String(err);
        // revert by re-fetching
        await this.fetchTasks();
        return null;
      }
    },

    async toggleTaskStatus(id: string): Promise<void> {
      const task = this.tasks.find((t) => t.id === id);
      if (!task) return;

      const nextStatus: TaskStatus = task.status === 'done' ? 'todo' : 'done';
      task.status = nextStatus;
      task.updated_at = Date.now();
      task.sync_status = 'pending';

      try {
        if (this.isTauriEnv) {
          const updated = await invoke<Task>('toggle_task_status', { id });
          const idx = this.tasks.findIndex((t) => t.id === id);
          if (idx !== -1) {
            this.tasks[idx] = updated;
          }
        }
      } catch (err: any) {
        console.error('Lỗi đổi trạng thái task:', err);
        this.errorMessage = String(err);
        await this.fetchTasks();
      }
    },

    async moveTaskStatus(id: string, newStatus: TaskStatus): Promise<void> {
      const task = this.tasks.find((t) => t.id === id);
      if (!task || task.status === newStatus) return;

      task.status = newStatus;
      task.updated_at = Date.now();
      task.sync_status = 'pending';

      try {
        if (this.isTauriEnv) {
          const updated = await invoke<Task>('move_task_status', {
            id,
            status: newStatus,
          });
          const idx = this.tasks.findIndex((t) => t.id === id);
          if (idx !== -1) {
            this.tasks[idx] = updated;
          }
        }
      } catch (err: any) {
        console.error('Lỗi di chuyển cột task:', err);
        this.errorMessage = String(err);
        await this.fetchTasks();
      }
    },

    async deleteTask(id: string): Promise<void> {
      // Remove from active list
      this.tasks = this.tasks.filter((t) => t.id !== id);
      if (this.selectedTaskId === id) {
        this.selectedTaskId = null;
      }

      try {
        if (this.isTauriEnv) {
          await invoke('delete_task', { id });
        }
      } catch (err: any) {
        console.error('Lỗi xóa task:', err);
        this.errorMessage = String(err);
        await this.fetchTasks();
      }
    },
  },
});
