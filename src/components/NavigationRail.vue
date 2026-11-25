<script setup lang="ts">
import { computed } from 'vue';
import { useTaskStore } from '@/stores/taskStore';
import {
  CheckSquare,
  Calendar,
  Search,
  RefreshCw,
  Sun,
  Moon,
  User,
  Power,
} from 'lucide-vue-next';

const store = useTaskStore();

const syncTooltip = computed(() => {
  if (store.syncTriggerLoading || store.syncState.state === 'syncing') {
    return 'Đang đồng bộ với Supabase Cloud...';
  }
  if (store.syncState.state === 'error') {
    return `Lỗi đồng bộ: ${store.syncState.error_message || 'Không thể kết nối'}`;
  }
  if (store.pendingSyncCount > 0) {
    return `Chờ đồng bộ: ${store.pendingSyncCount} task (Nhấp để gửi lên Cloud)`;
  }
  if (store.isLoggedIn) {
    return 'Đã đồng bộ an toàn với Supabase Cloud (Nhấp để đồng bộ lại)';
  }
  return 'Chế độ Local-first SQLite (Nhấp để kiểm tra đồng bộ)';
});
</script>

<template>
  <nav
    class="w-[54px] min-w-[54px] max-w-[54px] flex-shrink-0 flex flex-col items-center justify-between py-3.5 bg-slate-100/90 dark:bg-slate-950 border-r border-slate-200/80 dark:border-slate-800/80 select-none z-30 transition-colors"
  >
    <!-- Top: Logo & Main Navigation Icons -->
    <div class="flex flex-col items-center gap-3.5 w-full">
      <!-- App Brand Logo (Icon duy nhất của ứng dụng) -->
      <button
        type="button"
        @click="store.setNavTab('tasks')"
        class="w-9 h-9 rounded-xl bg-gradient-to-tr from-blue-600 to-indigo-500 flex items-center justify-center text-white shadow-sm hover:scale-105 transition-transform"
        title="TickTask Desktop"
      >
        <CheckSquare class="w-5 h-5" />
      </button>

      <!-- User Profile / Auth Button (TickTick style) -->
      <button
        type="button"
        @click="store.openAuthModal"
        :class="[
          'w-10 h-10 rounded-xl flex items-center justify-center transition-all relative group',
          store.isAuthModalOpen
            ? 'bg-blue-50 dark:bg-blue-950/60 text-blue-600 dark:text-blue-400 font-semibold shadow-xs'
            : 'text-slate-500 dark:text-slate-400 hover:bg-slate-200/60 dark:hover:bg-slate-800/60 hover:text-slate-800 dark:hover:text-slate-200'
        ]"
        :title="store.isLoggedIn ? `Tài khoản: ${store.currentUser?.email}` : 'Đăng nhập / Đồng bộ Cloud'"
      >
        <div
          v-if="store.isLoggedIn"
          class="w-7 h-7 rounded-full bg-gradient-to-tr from-blue-600 to-indigo-500 text-white font-bold text-xs flex items-center justify-center shadow-xs"
        >
          {{ store.currentUser?.email?.charAt(0).toUpperCase() }}
        </div>
        <User v-else class="w-5 h-5" />
        <span
          v-if="store.isLoggedIn"
          class="absolute bottom-1 right-1 w-2 h-2 rounded-full bg-emerald-500 ring-2 ring-white dark:ring-slate-900"
        ></span>
      </button>

      <div class="w-7 border-t border-slate-200 dark:border-slate-800 my-0.5"></div>

      <!-- Navigation Items: Tasks, Calendar, Search -->
      <div class="flex flex-col items-center gap-2 w-full px-1.5">
        <!-- Tasks Tab -->
        <button
          type="button"
          @click="store.setNavTab('tasks')"
          :class="[
            'w-10 h-10 rounded-xl flex items-center justify-center transition-all relative group',
            store.currentNavTab === 'tasks'
              ? 'bg-blue-50 dark:bg-blue-950/60 text-blue-600 dark:text-blue-400 font-semibold shadow-xs'
              : 'text-slate-500 dark:text-slate-400 hover:bg-slate-200/60 dark:hover:bg-slate-800/60 hover:text-slate-800 dark:hover:text-slate-200'
          ]"
          title="Công việc (Tasks)"
        >
          <CheckSquare class="w-5 h-5" />
          <span
            v-if="store.activeTasks.length > 0 && store.currentNavTab !== 'tasks'"
            class="absolute top-1 right-1 w-2 h-2 rounded-full bg-blue-500"
          ></span>
        </button>

        <!-- Calendar Tab (TickTick Blue highlight #3B82F6) -->
        <button
          type="button"
          @click="store.setNavTab('calendar')"
          :class="[
            'w-10 h-10 rounded-xl flex items-center justify-center transition-all relative group',
            store.currentNavTab === 'calendar'
              ? 'bg-[#3B82F6] text-white shadow-md shadow-blue-500/25'
              : 'text-slate-500 dark:text-slate-400 hover:bg-slate-200/60 dark:hover:bg-slate-800/60 hover:text-slate-800 dark:hover:text-slate-200'
          ]"
          title="Lịch biểu (Calendar View)"
        >
          <Calendar class="w-5 h-5" />
        </button>

        <!-- Search Tab (Mở Search Modal / Command Palette) -->
        <button
          type="button"
          @click="store.openSearchModal"
          :class="[
            'w-10 h-10 rounded-xl flex items-center justify-center transition-all relative group',
            store.isSearchModalOpen
              ? 'bg-blue-50 dark:bg-blue-950/60 text-blue-600 dark:text-blue-400 font-semibold shadow-xs'
              : 'text-slate-500 dark:text-slate-400 hover:bg-slate-200/60 dark:hover:bg-slate-800/60 hover:text-slate-800 dark:hover:text-slate-200'
          ]"
          title="Tìm kiếm toàn cục (Ctrl + K)"
        >
          <Search class="w-5 h-5" />
        </button>
      </div>
    </div>

    <!-- Bottom: Theme Toggle & Sync Status duy nhất -->
    <div class="flex flex-col items-center gap-2.5 w-full px-1.5">
      <!-- Nút chuyển đổi Light/Dark Mode -->
      <button
        type="button"
        @click="store.toggleTheme"
        class="w-9 h-9 rounded-lg flex items-center justify-center text-slate-500 dark:text-slate-400 hover:bg-slate-200/60 dark:hover:bg-slate-800/60 hover:text-slate-800 dark:hover:text-slate-200 transition-colors"
        :title="store.isDarkMode ? 'Chuyển sang giao diện Sáng' : 'Chuyển sang giao diện Tối'"
      >
        <Sun v-if="store.isDarkMode" class="w-4 h-4 text-amber-400" />
        <Moon v-else class="w-4 h-4 text-slate-500" />
      </button>

      <!-- Nút & Trạng thái Sync SQLite <-> Supabase Cloud -->
      <button
        type="button"
        @click="store.triggerSyncNow"
        class="w-9 h-9 rounded-lg flex items-center justify-center relative text-slate-500 dark:text-slate-400 hover:bg-slate-200/60 dark:hover:bg-slate-800/60 hover:text-slate-800 dark:hover:text-slate-200 transition-colors cursor-pointer"
        :class="store.syncTriggerLoading || store.syncState.state === 'syncing' ? 'text-blue-500' : ''"
        :title="syncTooltip"
      >
        <RefreshCw class="w-4 h-4" :class="store.syncTriggerLoading || store.syncState.state === 'syncing' ? 'animate-spin' : ''" />
        <span
          v-if="store.syncState.state === 'error'"
          class="absolute top-1 right-1 w-2 h-2 rounded-full bg-rose-500 ring-2 ring-white dark:ring-slate-900"
        ></span>
        <span
          v-else-if="store.pendingSyncCount > 0"
          class="absolute top-1 right-1 w-2 h-2 rounded-full bg-amber-500 ring-2 ring-white dark:ring-slate-900"
        ></span>
        <span
          v-else-if="store.isLoggedIn"
          class="absolute top-1 right-1 w-1.5 h-1.5 rounded-full bg-emerald-500"
        ></span>
      </button>

      <!-- Nút Thoát hoàn toàn ứng dụng -->
      <button
        type="button"
        @click="store.exitApp"
        class="w-9 h-9 rounded-lg flex items-center justify-center text-slate-400 hover:text-rose-600 dark:hover:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/40 transition-colors cursor-pointer"
        title="Thoát hẳn ứng dụng (Tắt chạy ngầm)"
      >
        <Power class="w-4 h-4" />
      </button>
    </div>
  </nav>
</template>
