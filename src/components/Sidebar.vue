<script setup lang="ts">
import { computed } from 'vue';
import { useTaskStore } from '@/stores/taskStore';
import type { FilterType } from '@/types/task';
import {
  Inbox,
  Calendar,
  AlertCircle,
  CheckCircle2,
  Database,
  Moon,
  Sun,
  PanelLeftClose,
  PanelLeft,
  CheckSquare,
} from 'lucide-vue-next';

const store = useTaskStore();

const filters: { id: FilterType; label: string; icon: any; colorClass: string }[] = [
  { id: 'all', label: 'Tất cả công việc', icon: Inbox, colorClass: 'text-blue-500' },
  { id: 'today', label: 'Hôm nay & Quá hạn', icon: Calendar, colorClass: 'text-amber-500' },
  { id: 'high_priority', label: 'Ưu tiên cao', icon: AlertCircle, colorClass: 'text-red-500' },
  { id: 'completed', label: 'Đã hoàn thành', icon: CheckCircle2, colorClass: 'text-emerald-500' },
];

const counts = computed(() => store.counts);
</script>

<template>
  <aside
    class="relative flex flex-col h-full bg-slate-50 dark:bg-slate-900/90 border-r border-slate-200 dark:border-slate-800 transition-all duration-200 ease-in-out select-none"
    :class="store.isSidebarCollapsed ? 'w-16' : 'w-[230px]'"
  >
    <!-- Brand / Header -->
    <div class="flex items-center justify-between px-3.5 py-4 border-b border-slate-200/60 dark:border-slate-800/80">
      <div v-if="!store.isSidebarCollapsed" class="flex items-center gap-2.5 overflow-hidden">
        <div class="w-8 h-8 rounded-lg bg-blue-600 flex items-center justify-center text-white shadow-sm flex-shrink-0">
          <CheckSquare class="w-5 h-5" />
        </div>
        <div class="flex flex-col">
          <span class="text-sm font-semibold tracking-tight text-slate-800 dark:text-slate-100 leading-tight">TickTask</span>
          <span class="text-[10px] text-slate-500 dark:text-slate-400">Local-first Workspace</span>
        </div>
      </div>
      <div v-else class="mx-auto">
        <div class="w-8 h-8 rounded-lg bg-blue-600 flex items-center justify-center text-white shadow-sm">
          <CheckSquare class="w-5 h-5" />
        </div>
      </div>

      <button
        @click="store.toggleSidebar"
        class="p-1.5 rounded-md text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-200/50 dark:hover:bg-slate-800 transition-colors"
        :title="store.isSidebarCollapsed ? 'Mở rộng sidebar' : 'Thu gọn sidebar'"
      >
        <PanelLeftClose v-if="!store.isSidebarCollapsed" class="w-4 h-4" />
        <PanelLeft v-else class="w-4 h-4" />
      </button>
    </div>

    <!-- Navigation / Filters -->
    <div class="flex-1 py-3 px-2 space-y-1 overflow-y-auto">
      <button
        v-for="item in filters"
        :key="item.id"
        @click="store.setFilter(item.id)"
        :class="[
          'w-full flex items-center gap-3 px-2.5 py-2 rounded-lg text-sm font-medium transition-all group relative',
          store.currentFilter === item.id
            ? 'bg-blue-50 dark:bg-blue-950/40 text-blue-600 dark:text-blue-400 shadow-sm'
            : 'text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800/60 hover:text-slate-900 dark:hover:text-slate-200'
        ]"
        :title="store.isSidebarCollapsed ? item.label : ''"
      >
        <component
          :is="item.icon"
          class="w-4 h-4 flex-shrink-0 transition-transform group-hover:scale-110"
          :class="item.colorClass"
        />

        <span
          v-if="!store.isSidebarCollapsed"
          class="flex-1 text-left truncate text-[13px]"
        >
          {{ item.label }}
        </span>

        <span
          v-if="!store.isSidebarCollapsed && counts[item.id] > 0"
          :class="[
            'px-1.5 py-0.5 text-xs font-semibold rounded-full',
            store.currentFilter === item.id
              ? 'bg-blue-100 dark:bg-blue-900/60 text-blue-700 dark:text-blue-300'
              : 'bg-slate-200/70 dark:bg-slate-800 text-slate-500 dark:text-slate-400'
          ]"
        >
          {{ counts[item.id] }}
        </span>

        <!-- Indicator dot when collapsed -->
        <span
          v-if="store.isSidebarCollapsed && counts[item.id] > 0"
          class="absolute top-1.5 right-1.5 w-2 h-2 rounded-full bg-blue-500"
        />
      </button>
    </div>

    <!-- Theme Switcher & System Controls -->
    <div class="px-2 py-2 border-t border-slate-200/60 dark:border-slate-800/80">
      <button
        @click="store.toggleTheme"
        class="w-full flex items-center gap-3 px-2.5 py-2 rounded-lg text-sm font-medium text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800/60 hover:text-slate-900 dark:hover:text-slate-200 transition-colors"
        :title="store.isDarkMode ? 'Chuyển sang giao diện Sáng' : 'Chuyển sang giao diện Tối'"
      >
        <Sun v-if="store.isDarkMode" class="w-4 h-4 text-amber-400 flex-shrink-0" />
        <Moon v-else class="w-4 h-4 text-slate-500 flex-shrink-0" />
        <span v-if="!store.isSidebarCollapsed" class="text-[13px]">
          {{ store.isDarkMode ? 'Chế độ Sáng' : 'Chế độ Tối' }}
        </span>
      </button>
    </div>

    <!-- Footer: Local-first SQLite Sync Status -->
    <div
      class="p-2.5 bg-slate-100/60 dark:bg-slate-950/60 border-t border-slate-200/60 dark:border-slate-800/80"
    >
      <div v-if="!store.isSidebarCollapsed" class="flex flex-col gap-1">
        <div class="flex items-center gap-2 text-slate-600 dark:text-slate-400">
          <Database class="w-3.5 h-3.5 text-blue-500" />
          <span class="text-[11px] font-medium truncate">SQLite Local-first</span>
        </div>
        <div class="flex items-center gap-1.5 pl-5">
          <span
            class="w-1.5 h-1.5 rounded-full"
            :class="store.pendingSyncCount > 0 ? 'bg-amber-400 animate-pulse' : 'bg-emerald-400'"
          />
          <span class="text-[10px] text-slate-500 dark:text-slate-400">
            Chờ đồng bộ: <strong class="text-slate-700 dark:text-slate-200">{{ store.pendingSyncCount }}</strong> task
          </span>
        </div>
      </div>
      <div v-else class="flex justify-center" :title="`SQLite Local-first: ${store.pendingSyncCount} task chờ sync`">
        <div class="relative">
          <Database class="w-4 h-4 text-blue-500" />
          <span
            v-if="store.pendingSyncCount > 0"
            class="absolute -top-1 -right-1 w-2 h-2 rounded-full bg-amber-400 animate-pulse"
          />
        </div>
      </div>
    </div>
  </aside>
</template>
