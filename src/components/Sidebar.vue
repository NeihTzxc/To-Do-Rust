<script setup lang="ts">
import { computed } from 'vue';
import { useTaskStore } from '@/stores/taskStore';
import type { FilterType } from '@/types/task';
import {
  Inbox,
  Calendar,
  AlertCircle,
  CheckCircle2,
} from 'lucide-vue-next';

const store = useTaskStore();

const filters: { id: FilterType; label: string; icon: any; colorClass: string }[] = [
  { id: 'all', label: 'Tất cả công việc', icon: Inbox, colorClass: 'text-blue-500' },
  { id: 'today', label: 'Hôm nay & Quá hạn', icon: Calendar, colorClass: 'text-amber-500' },
  { id: 'high_priority', label: 'Ưu tiên cao', icon: AlertCircle, colorClass: 'text-rose-500' },
  { id: 'completed', label: 'Đã hoàn thành', icon: CheckCircle2, colorClass: 'text-emerald-500' },
];

const counts = computed(() => store.counts);
</script>

<template>
  <aside
    class="w-[220px] min-w-[220px] max-w-[220px] flex-shrink-0 flex flex-col h-full bg-slate-50/90 dark:bg-slate-900/90 border-r border-slate-200/80 dark:border-slate-800 select-none transition-all duration-200 ease-in-out"
  >
    <!-- Header: Tiêu đề danh mục sạch sẽ, không trùng lặp logo -->
    <div class="px-4 py-3.5 border-b border-slate-200/60 dark:border-slate-800/80 flex items-center justify-between">
      <span class="text-xs font-bold tracking-wider uppercase text-slate-500 dark:text-slate-400">
        Bộ lọc thông minh
      </span>
      <span class="text-[11px] font-semibold px-2 py-0.5 rounded-full bg-slate-200/60 dark:bg-slate-800 text-slate-600 dark:text-slate-300">
        {{ counts.all }}
      </span>
    </div>

    <!-- Navigation / Filters List: Luôn hiển thị đầy đủ nhãn chữ và số lượng -->
    <div class="flex-1 py-3 px-2 space-y-1 overflow-y-auto">
      <button
        v-for="item in filters"
        :key="item.id"
        @click="store.setFilter(item.id)"
        :class="[
          'w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-all group relative',
          store.currentFilter === item.id
            ? 'bg-blue-50 dark:bg-blue-950/50 text-blue-600 dark:text-blue-400 font-semibold shadow-xs'
            : 'text-slate-600 dark:text-slate-400 hover:bg-slate-200/50 dark:hover:bg-slate-800/60 hover:text-slate-900 dark:hover:text-slate-200'
        ]"
      >
        <component
          :is="item.icon"
          class="w-4 h-4 flex-shrink-0 transition-transform group-hover:scale-110"
          :class="item.colorClass"
        />

        <span class="flex-1 text-left truncate text-[13px]">
          {{ item.label }}
        </span>

        <span
          v-if="counts[item.id] > 0"
          :class="[
            'px-2 py-0.5 text-xs font-semibold rounded-full',
            store.currentFilter === item.id
              ? 'bg-blue-100 dark:bg-blue-900/60 text-blue-700 dark:text-blue-300'
              : 'bg-slate-200/70 dark:bg-slate-800 text-slate-500 dark:text-slate-400'
          ]"
        >
          {{ counts[item.id] }}
        </span>
      </button>
    </div>

    <!-- Footer: Gọn gàng không chứa nút Dark Mode lặp lại -->
    <div class="p-3 border-t border-slate-200/60 dark:border-slate-800/80 text-[11px] text-slate-400 text-center">
      <span>TickTask v0.2.1 • Local-first</span>
    </div>
  </aside>
</template>
