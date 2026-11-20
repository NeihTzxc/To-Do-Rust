import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type {
  Task,
  TaskStatus,
  Priority,
  FilterType,
  ViewMode,
  CalendarViewMode,
  NavigationTab,
  CreateTaskPayload,
  UpdateTaskPayload,
  UserInfo,
  SyncStatusState,
} from '@/types/task';

function isTodayOrOverdue(timestamp: number | null): boolean {
  if (!timestamp) return false;
  const endOfToday = new Date();
  endOfToday.setHours(23, 59, 59, 999);
  return timestamp <= endOfToday.getTime();
}

// Fallback in-memory initial data for browser preview when tauri is not injected
const now = Date.now();
const todayDate = new Date();
const startOfToday = new Date(todayDate.getFullYear(), todayDate.getMonth(), todayDate.getDate()).getTime();

const mockTasks: Task[] = [
  {
    id: 'demo-1',
    title: 'Họp giao ban dự án & Kế hoạch tuần',
    description: 'Rà soát tiến độ tuần qua và phân bổ nhiệm vụ mới cho các thành viên.',
    status: 'in_progress',
    priority: 3,
    due_date: startOfToday + 18 * 3600000,
    start_time: startOfToday + 9 * 3600000, // 09:00
    end_time: startOfToday + 10.5 * 3600000, // 10:30
    created_at: now - 3600000,
    updated_at: now,
    deleted_at: null,
    sync_status: 'synced',
  },
  {
    id: 'demo-2',
    title: 'Thiết kế giao diện Calendar TickTick',
    description: 'Hoàn thiện Time-blocking Grid, Drag & Drop, Unscheduled Drawer.',
    status: 'todo',
    priority: 2,
    due_date: startOfToday + 20 * 3600000,
    start_time: startOfToday + 13 * 3600000, // 13:00
    end_time: startOfToday + 15 * 3600000, // 15:00
    created_at: now - 7200000,
    updated_at: now,
    deleted_at: null,
    sync_status: 'pending',
  },
  {
    id: 'demo-3',
    title: 'Kiểm tra code & Unit test Rust backend',
    description: 'Chạy cargo test và kiểm tra các câu truy vấn SQLite.',
    status: 'todo',
    priority: 1,
    due_date: startOfToday + 22 * 3600000,
    start_time: startOfToday + 16 * 3600000, // 16:00
    end_time: startOfToday + 17.5 * 3600000, // 17:30
    created_at: now - 86400000,
    updated_at: now,
    deleted_at: null,
    sync_status: 'synced',
  },
  {
    id: 'demo-4',
    title: 'Chuẩn bị tài liệu hướng dẫn sử dụng',
    description: 'Viết tài liệu chi tiết về cách dùng phím tắt và tính năng xếp lịch.',
    status: 'todo',
    priority: 2,
    due_date: null,
    start_time: null,
    end_time: null,
    created_at: now - 100000,
    updated_at: now,
    deleted_at: null,
    sync_status: 'pending',
  },
  {
    id: 'demo-5',
    title: 'Đánh giá bảo mật và mã hóa Local-first DB',
    description: 'Kiểm tra phân quyền thư mục sqlite và backup dữ liệu tự động.',
    status: 'todo',
    priority: 3,
    due_date: null,
    start_time: null,
    end_time: null,
    created_at: now - 200000,
    updated_at: now,
    deleted_at: null,
    sync_status: 'pending',
  },
];

export const useTaskStore = defineStore('tasks', {
  state: () => ({
    tasks: [] as Task[],
    selectedTaskId: null as string | null,
    currentNavTab: 'calendar' as NavigationTab,
    calendarViewMode: 'week' as CalendarViewMode,
    calendarCurrentDate: Date.now(),
    isUnscheduledOpen: true,
    isNightCollapsed: true,
    currentFilter: 'all' as FilterType,
    currentView: 'list' as ViewMode,
    searchQuery: '',
    isLoading: false,
    errorMessage: null as string | null,
    isDarkMode: false,
    isSidebarCollapsed: false,
    isSearchModalOpen: false,
    currentUser: null as UserInfo | null,
    syncState: {
      is_logged_in: false,
      user_email: null,
      state: 'idle',
      pending_count: 0,
      last_synced_at: null,
      error_message: null,
    } as SyncStatusState,
    isAuthModalOpen: false,
    authLoading: false,
    authError: null as string | null,
    syncTriggerLoading: false,
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

    unscheduledTasks: (state): Task[] => {
      return state.tasks.filter((t) => t.status !== 'done' && (!t.start_time || !t.end_time));
    },

    scheduledTasks: (state): Task[] => {
      return state.tasks.filter((t) => !!(t.start_time && t.end_time));
    },

    pendingSyncCount: (state): number => {
      return state.syncState.pending_count || state.tasks.filter((t) => t.sync_status === 'pending').length;
    },

    isLoggedIn: (state): boolean => {
      return !!state.currentUser;
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
        this.isDarkMode = false; // default Light mode
      }
      this.applyTheme();

      await this.initAuthAndSync();
      await this.fetchTasks();
    },

    async initAuthAndSync() {
      if (this.isTauriEnv) {
        try {
          const user = await invoke<UserInfo | null>('get_current_user');
          this.currentUser = user;
          const status = await invoke<SyncStatusState>('get_sync_status');
          this.syncState = status;

          await listen('tasks-synced', async () => {
            console.log('[Sync] Event tasks-synced received from Rust backend, refreshing local tasks...');
            await this.fetchTasks();
            await this.fetchSyncStatus();
          });
        } catch (e) {
          console.warn('Init auth/sync warning:', e);
        }
      }
    },

    async fetchSyncStatus() {
      if (this.isTauriEnv) {
        try {
          const status = await invoke<SyncStatusState>('get_sync_status');
          this.syncState = status;
        } catch (e) {
          console.warn('fetchSyncStatus error:', e);
        }
      }
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

    setNavTab(tab: NavigationTab) {
      this.currentNavTab = tab;
    },

    setCalendarViewMode(mode: CalendarViewMode) {
      this.calendarViewMode = mode;
    },

    setCalendarDate(timestamp: number) {
      this.calendarCurrentDate = timestamp;
    },

    prevCalendarPeriod() {
      const d = new Date(this.calendarCurrentDate);
      if (this.calendarViewMode === 'day') {
        d.setDate(d.getDate() - 1);
      } else if (this.calendarViewMode === 'week') {
        d.setDate(d.getDate() - 7);
      } else if (this.calendarViewMode === 'month') {
        d.setMonth(d.getMonth() - 1);
      } else {
        d.setDate(d.getDate() - 7);
      }
      this.calendarCurrentDate = d.getTime();
    },

    nextCalendarPeriod() {
      const d = new Date(this.calendarCurrentDate);
      if (this.calendarViewMode === 'day') {
        d.setDate(d.getDate() + 1);
      } else if (this.calendarViewMode === 'week') {
        d.setDate(d.getDate() + 7);
      } else if (this.calendarViewMode === 'month') {
        d.setMonth(d.getMonth() + 1);
      } else {
        d.setDate(d.getDate() + 7);
      }
      this.calendarCurrentDate = d.getTime();
    },

    todayCalendarPeriod() {
      this.calendarCurrentDate = Date.now();
    },

    toggleUnscheduled() {
      this.isUnscheduledOpen = !this.isUnscheduledOpen;
    },

    toggleNightCollapsed() {
      this.isNightCollapsed = !this.isNightCollapsed;
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
            startTime: payload.start_time ?? null,
            endTime: payload.end_time ?? null,
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
            start_time: payload.start_time ?? null,
            end_time: payload.end_time ?? null,
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
          start_time: payload.start_time !== undefined ? payload.start_time : this.tasks[idx].start_time,
          end_time: payload.end_time !== undefined ? payload.end_time : this.tasks[idx].end_time,
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
            startTime: payload.start_time !== undefined ? payload.start_time : (this.tasks[idx]?.start_time ?? null),
            endTime: payload.end_time !== undefined ? payload.end_time : (this.tasks[idx]?.end_time ?? null),
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
        await this.fetchTasks();
        return null;
      }
    },

    async updateTaskSchedule(id: string, startTime: number | null, endTime: number | null): Promise<void> {
      const task = this.tasks.find((t) => t.id === id);
      if (!task) return;

      task.start_time = startTime;
      task.end_time = endTime;
      task.updated_at = Date.now();
      task.sync_status = 'pending';

      try {
        if (this.isTauriEnv) {
          const updated = await invoke<Task>('update_task_schedule', {
            id,
            startTime,
            endTime,
          });
          const idx = this.tasks.findIndex((t) => t.id === id);
          if (idx !== -1) {
            this.tasks[idx] = updated;
          }
        }
      } catch (err: any) {
        console.error('Lỗi cập nhật lịch task:', err);
        this.errorMessage = String(err);
        await this.fetchTasks();
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

    openSearchModal() {
      this.isSearchModalOpen = true;
    },

    closeSearchModal() {
      this.isSearchModalOpen = false;
    },

    toggleSearchModal() {
      this.isSearchModalOpen = !this.isSearchModalOpen;
    },

    async searchTasksRemote(query: string): Promise<Task[]> {
      try {
        if (this.isTauriEnv) {
          return await invoke<Task[]>('search_tasks', { query });
        } else {
          const q = query.trim().toLowerCase();
          if (!q) {
            return this.tasks.slice(0, 8);
          }
          const removeAccents = (str: string) =>
            str.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase();
          const normQ = removeAccents(q);
          return this.tasks
            .filter((t) => {
              const normTitle = removeAccents(t.title);
              const normDesc = t.description ? removeAccents(t.description) : '';
              return normTitle.includes(normQ) || normDesc.includes(normQ);
            })
            .sort((a, b) => {
              const doneA = a.status === 'done' ? 1 : 0;
              const doneB = b.status === 'done' ? 1 : 0;
              return doneA - doneB || b.updated_at - a.updated_at;
            })
            .slice(0, 50);
        }
      } catch (err: any) {
        console.error('Lỗi tìm kiếm task:', err);
        return [];
      }
    },

    openAuthModal() {
      this.isAuthModalOpen = true;
      this.authError = null;
    },

    closeAuthModal() {
      this.isAuthModalOpen = false;
      this.authError = null;
    },

    toggleAuthModal() {
      this.isAuthModalOpen = !this.isAuthModalOpen;
      this.authError = null;
    },

    async login(email: string, password: string): Promise<boolean> {
      this.authLoading = true;
      this.authError = null;
      try {
        if (this.isTauriEnv) {
          const user = await invoke<UserInfo>('login', { email, password });
          this.currentUser = user;
          await this.fetchTasks();
          await this.fetchSyncStatus();
        } else {
          this.currentUser = { user_id: 'mock-user-1', email };
          this.syncState.is_logged_in = true;
          this.syncState.user_email = email;
          this.syncState.state = 'synced';
        }
        this.closeAuthModal();
        return true;
      } catch (err: any) {
        console.error('Lỗi đăng nhập:', err);
        this.authError = String(err);
        return false;
      } finally {
        this.authLoading = false;
      }
    },

    async register(email: string, password: string): Promise<boolean> {
      this.authLoading = true;
      this.authError = null;
      try {
        if (this.isTauriEnv) {
          const user = await invoke<UserInfo>('register', { email, password });
          this.currentUser = user;
          await this.fetchTasks();
          await this.fetchSyncStatus();
        } else {
          this.currentUser = { user_id: 'mock-user-1', email };
          this.syncState.is_logged_in = true;
          this.syncState.user_email = email;
          this.syncState.state = 'synced';
        }
        this.closeAuthModal();
        return true;
      } catch (err: any) {
        console.error('Lỗi đăng ký:', err);
        this.authError = String(err);
        return false;
      } finally {
        this.authLoading = false;
      }
    },

    async logout(): Promise<void> {
      this.authLoading = true;
      this.authError = null;
      try {
        if (this.isTauriEnv) {
          await invoke('logout');
        }
        this.currentUser = null;
        this.syncState.is_logged_in = false;
        this.syncState.user_email = null;
        this.syncState.state = 'idle';
        await this.fetchTasks();
        await this.fetchSyncStatus();
      } catch (err: any) {
        console.error('Lỗi đăng xuất:', err);
        this.authError = String(err);
      } finally {
        this.authLoading = false;
      }
    },

    async triggerSyncNow(): Promise<void> {
      if (this.syncTriggerLoading) return;
      this.syncTriggerLoading = true;
      try {
        if (this.isTauriEnv) {
          await invoke('trigger_sync_now');
          await this.fetchSyncStatus();
        }
      } catch (err: any) {
        console.error('Lỗi kích hoạt sync:', err);
      } finally {
        setTimeout(async () => {
          this.syncTriggerLoading = false;
          await this.fetchSyncStatus();
        }, 1200);
      }
    },
  },
});
