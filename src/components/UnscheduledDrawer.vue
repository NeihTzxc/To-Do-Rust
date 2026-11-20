<script setup lang="ts">
import { ref, computed } from 'vue';
import { useTaskStore } from '@/stores/taskStore';
import type { Task, Priority } from '@/types/task';
import {
  ListPlus,
  Check,
  PanelLeftClose,
  Flag,
  Calendar,
  GripVertical,
} from 'lucide-vue-next';

const store = useTaskStore();
const quickTitle = ref('');
const searchFilter = ref('');

const unscheduledList = computed(() => {
  let list = store.unscheduledTasks;
  if (searchFilter.value.trim()) {
    const q = searchFilter.value.toLowerCase();
    list = list.filter((t) => t.title.toLowerCase().includes(q));
  }
  return list;
});

async function handleQuickAdd() {
  const val = quickTitle.value.trim();
  if (!val) return;
  await store.createTask({
    title: val,
    priority: 2,
    due_date: null,
    start_time: null,
    end_time: null,
  });
  quickTitle.value = '';
}

function handleDragStart(e: DragEvent, task: Task) {
  if (e.dataTransfer) {
    e.dataTransfer.setData('application/json', JSON.stringify({ taskId: task.id, type: 'unscheduled' }));
    e.dataTransfer.setData('text/plain', task.id);
    e.dataTransfer.effectAllowed = 'move';
  }
}

function getPriorityColor(p: Priority): string {
  switch (p) {
    case 3:
      return 'border-l-rose-500 bg-rose-50/40 dark:bg-rose-950/20';
    case 2:
      return 'border-l-blue-500 bg-blue-50/40 dark:bg-blue-950/20';
    case 1:
      return 'border-l-teal-500 bg-teal-50/40 dark:bg-teal-950/20';
    default:
      return 'border-l-slate-400 bg-slate-50 dark:bg-slate-800/40';
  }
}
</script>

<template>
  <aside
    v-if="store.isUnscheduledOpen"
    class="w-64 flex-shrink-0 flex flex-col h-full bg-slate-50/90 dark:bg-slate-900/90 border-r border-slate-200/80 dark:border-slate-800/80 transition-all select-none z-10"
  >
    <!-- Drawer Header -->
    <div class="flex items-center justify-between px-3.5 py-3 border-b border-slate-200/70 dark:border-slate-800/80">
      <div class="flex items-center gap-2">
        <h3 class="text-xs font-bold uppercase tracking-wider text-slate-700 dark:text-slate-200">
          Chưa xếp lịch
        </h3>
        <span
          class="px-1.5 py-0.5 text-[10px] font-semibold rounded-full bg-slate-200/80 dark:bg-slate-800 text-slate-600 dark:text-slate-300"
        >
          {{ unscheduledList.length }}
        </span>
      </div>

      <button
        type="button"
        @click="store.toggleUnscheduled"
        class="p-1 rounded-md text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200/50 dark:hover:bg-slate-800 transition-colors"
        title="Thu gọn danh sách chưa xếp lịch"
      >
        <PanelLeftClose class="w-4 h-4" />
      </button>
    </div>

    <!-- Quick Add Input -->
    <div class="p-2.5 border-b border-slate-200/60 dark:border-slate-800/60">
      <form @submit.prevent="handleQuickAdd" class="relative">
        <input
          v-model="quickTitle"
          type="text"
          placeholder="+ Thêm task chưa xếp lịch..."
          class="w-full pl-3 pr-8 py-1.5 text-xs bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-slate-800 dark:text-slate-100 placeholder-slate-400 outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500 transition-all shadow-sm"
        />
        <button
          v-if="quickTitle"
          type="submit"
          class="absolute right-2 top-1/2 -translate-y-1/2 text-blue-500 hover:text-blue-600 p-0.5"
        >
          <ListPlus class="w-3.5 h-3.5" />
        </button>
      </form>
    </div>

    <!-- List of Draggable Unscheduled Tasks -->
    <div class="flex-1 p-2 space-y-1.5 overflow-y-auto">
      <div
        v-if="unscheduledList.length === 0"
        class="flex flex-col items-center justify-center h-48 text-center px-4"
      >
        <Calendar class="w-8 h-8 text-slate-300 dark:text-slate-600 mb-2" />
        <p class="text-xs text-slate-400 dark:text-slate-500 font-medium">
          Không có task nào chưa xếp lịch
        </p>
      </div>

      <div
        v-for="task in unscheduledList"
        :key="task.id"
        draggable="true"
        @dragstart="handleDragStart($event, task)"
        @click="store.selectTask(task.id)"
        :class="[
          'group flex items-start gap-2 p-2 rounded-lg border border-slate-200/80 dark:border-slate-800 border-l-4 cursor-grab active:cursor-grabbing hover:shadow-md transition-all',
          getPriorityColor(task.priority),
          store.selectedTaskId === task.id ? 'ring-2 ring-blue-500/60' : ''
        ]"
      >
        <!-- Drag Handle Indicator -->
        <GripVertical class="w-3.5 h-3.5 text-slate-300 dark:text-slate-600 group-hover:text-slate-400 flex-shrink-0 mt-0.5" />

        <!-- Checkbox -->
        <button
          type="button"
          @click.stop="store.toggleTaskStatus(task.id)"
          :class="[
            'w-4 h-4 rounded border flex items-center justify-center transition-colors flex-shrink-0 mt-0.5',
            task.status === 'done'
              ? 'bg-emerald-500 border-emerald-500 text-white'
              : 'border-slate-300 dark:border-slate-600 hover:border-blue-500 text-transparent'
          ]"
        >
          <Check class="w-3 h-3 stroke-[2.5]" />
        </button>

        <!-- Task Content -->
        <div class="flex-1 min-w-0">
          <p class="text-xs font-semibold text-slate-800 dark:text-slate-100 truncate leading-snug">
            {{ task.title }}
          </p>
          <div class="flex items-center gap-2 mt-1 text-[10px] text-slate-400">
            <span v-if="task.priority === 3" class="text-rose-500 font-medium flex items-center gap-0.5">
              <Flag class="w-2.5 h-2.5 fill-current" /> Cao
            </span>
            <span v-else-if="task.priority === 2" class="text-blue-500 font-medium">
              Vừa
            </span>
            <span v-else class="text-teal-500 font-medium">
              Thấp
            </span>

            <span v-if="task.due_date" class="truncate">
              Hạn: {{ new Date(task.due_date).toLocaleDateString('vi-VN', { day: '2-digit', month: '2-digit' }) }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Drag Hint Footer -->
    <div class="p-2.5 bg-slate-100/60 dark:bg-slate-950/60 border-t border-slate-200/60 dark:border-slate-800/80 text-[11px] text-slate-400 text-center">
      Kéo task thả vào khung giờ trên lịch để time-block
    </div>
  </aside>
</template>
