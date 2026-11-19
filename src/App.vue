<script setup lang="ts">
import { onMounted, onUnmounted, computed } from 'vue';
import { useTaskStore } from '@/stores/taskStore';
import Sidebar from '@/components/Sidebar.vue';
import QuickAddBar from '@/components/QuickAddBar.vue';
import TaskListView from '@/components/TaskListView.vue';
import TaskKanbanView from '@/components/TaskKanbanView.vue';
import TaskDetailPanel from '@/components/TaskDetailPanel.vue';
import {
  List,
  Kanban,
  Search,
  X,
  RefreshCw,
} from 'lucide-vue-next';

const store = useTaskStore();

onMounted(async () => {
  await store.init();
  window.addEventListener('keydown', handleGlobalKeydown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleGlobalKeydown);
});

function handleGlobalKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && store.selectedTaskId) {
    store.selectTask(null);
  }
}

const currentFilterTitle = computed(() => {
  switch (store.currentFilter) {
    case 'today':
      return 'Hôm nay & Quá hạn';
    case 'high_priority':
      return 'Ưu tiên cao';
    case 'completed':
      return 'Đã hoàn thành';
    case 'all':
    default:
      return 'Tất cả công việc';
  }
});
</script>

<template>
  <div class="flex h-screen w-screen overflow-hidden bg-slate-50 dark:bg-slate-900 text-slate-800 dark:text-slate-100 font-sans">
    <!-- Cột 1: Sidebar -->
    <Sidebar />

    <!-- Cột 2: Khu vực làm việc chính -->
    <main class="flex-1 flex flex-col min-w-0 h-full overflow-hidden bg-white dark:bg-slate-900/60">
      <!-- Header bar -->
      <header class="flex items-center justify-between px-6 py-3.5 border-b border-slate-200/80 dark:border-slate-800 gap-4 flex-shrink-0">
        <!-- Filter Title -->
        <div class="flex items-center gap-3 min-w-0">
          <h1 class="text-lg font-bold tracking-tight text-slate-800 dark:text-slate-100 truncate">
            {{ currentFilterTitle }}
          </h1>
          <button
            @click="store.fetchTasks"
            class="p-1 rounded-md text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
            :class="store.isLoading ? 'animate-spin text-blue-500' : ''"
            title="Làm mới dữ liệu từ SQLite"
          >
            <RefreshCw class="w-3.5 h-3.5" />
          </button>
        </div>

        <!-- Controls: Search & View Switcher -->
        <div class="flex items-center gap-3 flex-shrink-0">
          <!-- Search Input -->
          <div class="relative w-44 md:w-56">
            <Search class="w-3.5 h-3.5 absolute left-2.5 top-1/2 -translate-y-1/2 text-slate-400" />
            <input
              v-model="store.searchQuery"
              type="text"
              placeholder="Tìm kiếm công việc..."
              class="w-full pl-8 pr-7 py-1 text-xs bg-slate-100 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-slate-800 dark:text-slate-200 outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500 transition-all placeholder-slate-400"
            />
            <button
              v-if="store.searchQuery"
              @click="store.searchQuery = ''"
              class="absolute right-2 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200"
            >
              <X class="w-3 h-3" />
            </button>
          </div>

          <!-- View Mode Toggle: [ Danh sách | Bảng Kanban ] -->
          <div class="flex items-center bg-slate-100 dark:bg-slate-800 p-0.5 rounded-lg border border-slate-200/60 dark:border-slate-700/60">
            <button
              @click="store.setView('list')"
              class="flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-md transition-all"
              :class="
                store.currentView === 'list'
                  ? 'bg-white dark:bg-slate-700 text-blue-600 dark:text-blue-400 shadow-sm font-semibold'
                  : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'
              "
              title="Chế độ Danh sách"
            >
              <List class="w-3.5 h-3.5" />
              <span class="hidden sm:inline">Danh sách</span>
            </button>

            <button
              @click="store.setView('kanban')"
              class="flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-md transition-all"
              :class="
                store.currentView === 'kanban'
                  ? 'bg-white dark:bg-slate-700 text-blue-600 dark:text-blue-400 shadow-sm font-semibold'
                  : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'
              "
              title="Chế độ Bảng Kanban"
            >
              <Kanban class="w-3.5 h-3.5" />
              <span class="hidden sm:inline">Bảng Kanban</span>
            </button>
          </div>
        </div>
      </header>

      <!-- Center Content Area -->
      <div class="flex-1 flex flex-col p-6 min-h-0 overflow-hidden">
        <!-- Quick Add Bar -->
        <QuickAddBar />

        <!-- Main View Switching -->
        <TaskListView v-if="store.currentView === 'list'" />
        <TaskKanbanView v-else-if="store.currentView === 'kanban'" />
      </div>
    </main>

    <!-- Cột 3: Task Detail Drawer -->
    <TaskDetailPanel />
  </div>
</template>
