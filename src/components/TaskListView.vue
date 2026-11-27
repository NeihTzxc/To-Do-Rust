<script setup lang="ts">
import { ref, computed } from 'vue';
import { useTaskStore } from '@/stores/taskStore';
import type { Priority } from '@/types/task';
import { formatRecurrenceLabel } from '@/utils/calendarTask';
import {
  ChevronDown,
  ChevronRight,
  Calendar,
  Check,
  Clock,
  Circle,
  FileText,
  Repeat,
} from 'lucide-vue-next';

const store = useTaskStore();

const isCompletedExpanded = ref(false);

const inProgressTasks = computed(() =>
  store.filteredTasks.filter((t) => t.status === 'in_progress')
);

const todoTasks = computed(() =>
  store.filteredTasks.filter((t) => t.status === 'todo')
);

const completedTasks = computed(() =>
  store.filteredTasks.filter((t) => t.status === 'done')
);

function getPriorityBorderColor(priority: Priority): string {
  switch (priority) {
    case 3:
      return 'border-red-500 hover:bg-red-500/10 text-red-500';
    case 2:
      return 'border-amber-500 hover:bg-amber-500/10 text-amber-500';
    case 1:
      return 'border-blue-500 hover:bg-blue-500/10 text-blue-500';
    default:
      return 'border-slate-400 hover:bg-slate-400/10 text-slate-400';
  }
}

function formatTaskDate(timestamp: number | null): { text: string; isOverdue: boolean } | null {
  if (!timestamp) return null;
  const d = new Date(timestamp);
  const now = new Date();
  const todayEnd = new Date();
  todayEnd.setHours(23, 59, 59, 999);

  const isOverdue = timestamp < now.getTime() && !(
    d.getDate() === now.getDate() &&
    d.getMonth() === now.getMonth() &&
    d.getFullYear() === now.getFullYear()
  );

  const isSameDay =
    d.getDate() === now.getDate() &&
    d.getMonth() === now.getMonth() &&
    d.getFullYear() === now.getFullYear();

  const tomorrow = new Date();
  tomorrow.setDate(now.getDate() + 1);
  const isTomorrow =
    d.getDate() === tomorrow.getDate() &&
    d.getMonth() === tomorrow.getMonth() &&
    d.getFullYear() === tomorrow.getFullYear();

  if (isSameDay) return { text: 'Hôm nay', isOverdue: false };
  if (isTomorrow) return { text: 'Ngày mai', isOverdue: false };
  return {
    text: `${d.getDate()}/${d.getMonth() + 1}`,
    isOverdue,
  };
}
</script>

<template>
  <div class="flex-1 overflow-y-auto space-y-6 pb-12 select-none">
    <!-- Empty state -->
    <div
      v-if="store.filteredTasks.length === 0"
      class="flex flex-col items-center justify-center py-16 text-center text-slate-400 dark:text-slate-500"
    >
      <div class="w-16 h-16 rounded-2xl bg-slate-100 dark:bg-slate-800/60 flex items-center justify-center mb-3 text-slate-400">
        <Circle class="w-8 h-8" />
      </div>
      <p class="text-sm font-medium text-slate-600 dark:text-slate-400">Không có công việc nào ở mục này</p>
      <p class="text-xs text-slate-400 mt-1">Dùng thanh nhập ở trên để thêm công việc mới nhanh chóng</p>
    </div>

    <!-- Section: In Progress -->
    <div v-if="inProgressTasks.length > 0" class="space-y-1.5">
      <div class="flex items-center gap-2 px-1 text-xs font-semibold text-blue-600 dark:text-blue-400 uppercase tracking-wider">
        <Clock class="w-3.5 h-3.5" />
        <span>Đang thực hiện ({{ inProgressTasks.length }})</span>
      </div>
      <div class="divide-y divide-slate-100 dark:divide-slate-800/80 bg-white dark:bg-slate-800/50 rounded-xl border border-slate-200/80 dark:border-slate-800 shadow-sm overflow-hidden">
        <div
          v-for="task in inProgressTasks"
          :key="task.id"
          @click="store.selectTask(task.id)"
          :class="[
            'group flex items-center gap-3 px-3.5 py-2.5 cursor-pointer transition-colors',
            store.selectedTaskId === task.id
              ? 'bg-blue-50/80 dark:bg-blue-950/40 text-blue-900 dark:text-blue-100'
              : 'hover:bg-slate-50 dark:hover:bg-slate-700/40'
          ]"
        >
          <!-- TickTick-styled round checkbox with priority-colored border -->
          <button
            type="button"
            @click.stop="store.toggleTaskStatus(task.id)"
            class="group/check w-5 h-5 flex items-center justify-center flex-shrink-0 cursor-pointer"
            title="Đánh dấu hoàn thành"
          >
            <span
              :class="[
                'w-3.5 h-3.5 rounded-full border-[1.5px] flex items-center justify-center transition-all group-hover/check:scale-110',
                getPriorityBorderColor(task.priority),
                task.status === 'done' ? 'bg-blue-500 border-blue-500 text-white' : ''
              ]"
            >
              <Check v-if="task.status === 'done'" class="w-2.5 h-2.5 stroke-[2.5]" />
            </span>
          </button>

          <!-- Task Title & Snippet -->
          <div class="flex-1 min-w-0">
            <span
              class="text-sm font-medium text-slate-800 dark:text-slate-200 truncate block"
              :class="task.status === 'done' ? 'line-through text-slate-400 dark:text-slate-500' : ''"
            >
              {{ task.title }}
            </span>
          </div>

          <!-- Recurrence badge -->
          <div
            v-if="task.recurrence_rule"
            class="flex items-center gap-1 text-[11px] px-1.5 py-0.5 rounded-md bg-indigo-50 dark:bg-indigo-950/40 text-indigo-600 dark:text-indigo-300 flex-shrink-0"
            title="Công việc lặp lại"
          >
            <Repeat class="w-3 h-3" />
            <span>{{ formatRecurrenceLabel(task.recurrence_rule) }}</span>
          </div>

          <!-- Description badge icon if present -->
          <div v-if="task.description" class="text-slate-400" title="Có ghi chú chi tiết">
            <FileText class="w-3.5 h-3.5" />
          </div>

          <!-- Due date badge -->
          <div
            v-if="formatTaskDate(task.due_date)"
            class="flex items-center gap-1 text-xs px-2 py-0.5 rounded-md flex-shrink-0"
            :class="
              formatTaskDate(task.due_date)?.isOverdue
                ? 'text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-950/40 font-medium'
                : 'text-slate-500 dark:text-slate-400 bg-slate-100 dark:bg-slate-700/50'
            "
          >
            <Calendar class="w-3 h-3" />
            <span>{{ formatTaskDate(task.due_date)?.text }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Section: Todo -->
    <div v-if="todoTasks.length > 0" class="space-y-1.5">
      <div class="flex items-center gap-2 px-1 text-xs font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider">
        <span>Cần làm ({{ todoTasks.length }})</span>
      </div>
      <div class="divide-y divide-slate-100 dark:divide-slate-800/80 bg-white dark:bg-slate-800/50 rounded-xl border border-slate-200/80 dark:border-slate-800 shadow-sm overflow-hidden">
        <div
          v-for="task in todoTasks"
          :key="task.id"
          @click="store.selectTask(task.id)"
          :class="[
            'group flex items-center gap-3 px-3.5 py-2.5 cursor-pointer transition-colors',
            store.selectedTaskId === task.id
              ? 'bg-blue-50/80 dark:bg-blue-950/40 text-blue-900 dark:text-blue-100'
              : 'hover:bg-slate-50 dark:hover:bg-slate-700/40'
          ]"
        >
          <!-- TickTick-styled round checkbox with priority-colored border -->
          <button
            type="button"
            @click.stop="store.toggleTaskStatus(task.id)"
            class="group/check w-5 h-5 flex items-center justify-center flex-shrink-0 cursor-pointer"
            title="Đánh dấu hoàn thành"
          >
            <span
              :class="[
                'w-3.5 h-3.5 rounded-full border-[1.5px] flex items-center justify-center transition-all group-hover/check:scale-110',
                getPriorityBorderColor(task.priority),
                task.status === 'done' ? 'bg-blue-500 border-blue-500 text-white' : ''
              ]"
            >
              <Check v-if="task.status === 'done'" class="w-2.5 h-2.5 stroke-[2.5]" />
            </span>
          </button>

          <!-- Task Title & Snippet -->
          <div class="flex-1 min-w-0">
            <span
              class="text-sm font-medium text-slate-800 dark:text-slate-200 truncate block"
              :class="task.status === 'done' ? 'line-through text-slate-400 dark:text-slate-500' : ''"
            >
              {{ task.title }}
            </span>
          </div>

          <!-- Recurrence badge -->
          <div
            v-if="task.recurrence_rule"
            class="flex items-center gap-1 text-[11px] px-1.5 py-0.5 rounded-md bg-indigo-50 dark:bg-indigo-950/40 text-indigo-600 dark:text-indigo-300 flex-shrink-0"
            title="Công việc lặp lại"
          >
            <Repeat class="w-3 h-3" />
            <span>{{ formatRecurrenceLabel(task.recurrence_rule) }}</span>
          </div>

          <!-- Description badge icon if present -->
          <div v-if="task.description" class="text-slate-400" title="Có ghi chú chi tiết">
            <FileText class="w-3.5 h-3.5" />
          </div>

          <!-- Due date badge -->
          <div
            v-if="formatTaskDate(task.due_date)"
            class="flex items-center gap-1 text-xs px-2 py-0.5 rounded-md flex-shrink-0"
            :class="
              formatTaskDate(task.due_date)?.isOverdue
                ? 'text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-950/40 font-medium'
                : 'text-slate-500 dark:text-slate-400 bg-slate-100 dark:bg-slate-700/50'
            "
          >
            <Calendar class="w-3 h-3" />
            <span>{{ formatTaskDate(task.due_date)?.text }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Section: Completed (Collapsible) -->
    <div v-if="completedTasks.length > 0" class="space-y-1.5 pt-2">
      <button
        @click="isCompletedExpanded = !isCompletedExpanded"
        class="flex items-center gap-2 px-1 text-xs font-semibold text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 uppercase tracking-wider transition-colors"
      >
        <component :is="isCompletedExpanded ? ChevronDown : ChevronRight" class="w-3.5 h-3.5" />
        <span>Đã hoàn thành ({{ completedTasks.length }})</span>
      </button>

      <div
        v-if="isCompletedExpanded"
        class="divide-y divide-slate-100 dark:divide-slate-800/80 bg-white/60 dark:bg-slate-800/30 rounded-xl border border-slate-200/60 dark:border-slate-800/60 shadow-sm overflow-hidden"
      >
        <div
          v-for="task in completedTasks"
          :key="task.id"
          @click="store.selectTask(task.id)"
          :class="[
            'group flex items-center gap-3 px-3.5 py-2.5 cursor-pointer transition-colors opacity-75 hover:opacity-100',
            store.selectedTaskId === task.id
              ? 'bg-blue-50/80 dark:bg-blue-950/40'
              : 'hover:bg-slate-50 dark:hover:bg-slate-700/40'
          ]"
        >
          <!-- TickTick-styled round checkbox with check -->
          <button
            type="button"
            @click.stop="store.toggleTaskStatus(task.id)"
            class="group/check w-5 h-5 flex items-center justify-center flex-shrink-0 cursor-pointer"
            title="Đánh dấu chưa hoàn thành"
          >
            <span class="w-3.5 h-3.5 rounded-full border-[1.5px] border-emerald-500 bg-emerald-500 text-white flex items-center justify-center transition-all group-hover/check:scale-110">
              <Check class="w-2.5 h-2.5 stroke-[2.5]" />
            </span>
          </button>

          <!-- Task Title -->
          <div class="flex-1 min-w-0">
            <span class="text-sm font-medium text-slate-400 dark:text-slate-500 line-through truncate block">
              {{ task.title }}
            </span>
          </div>

          <!-- Recurrence badge -->
          <div
            v-if="task.recurrence_rule"
            class="flex items-center gap-1 text-[11px] px-1.5 py-0.5 rounded-md bg-indigo-50 dark:bg-indigo-950/40 text-indigo-600 dark:text-indigo-300 flex-shrink-0"
            title="Công việc lặp lại"
          >
            <Repeat class="w-3 h-3" />
            <span>{{ formatRecurrenceLabel(task.recurrence_rule) }}</span>
          </div>

          <!-- Description badge icon if present -->
          <div v-if="task.description" class="text-slate-400" title="Có ghi chú chi tiết">
            <FileText class="w-3.5 h-3.5" />
          </div>

          <!-- Due date badge -->
          <div
            v-if="formatTaskDate(task.due_date)"
            class="flex items-center gap-1 text-xs px-2 py-0.5 rounded-md text-slate-400 bg-slate-100/50 dark:bg-slate-800/50 flex-shrink-0"
          >
            <Calendar class="w-3 h-3" />
            <span>{{ formatTaskDate(task.due_date)?.text }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
