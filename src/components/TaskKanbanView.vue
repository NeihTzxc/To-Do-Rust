<script setup lang="ts">
import { ref, computed } from 'vue';
import { useTaskStore } from '@/stores/taskStore';
import type { Task, TaskStatus, Priority } from '@/types/task';
import {
  Calendar,
  Check,
  Circle,
  Clock,
  CheckCircle2,
  FileText,
  GripVertical,
} from 'lucide-vue-next';

const store = useTaskStore();

const columns: { id: TaskStatus; title: string; icon: any; colorClass: string; badgeBg: string }[] = [
  {
    id: 'todo',
    title: 'Cần làm',
    icon: Circle,
    colorClass: 'text-slate-600 dark:text-slate-300',
    badgeBg: 'bg-slate-200 dark:bg-slate-700 text-slate-700 dark:text-slate-300',
  },
  {
    id: 'in_progress',
    title: 'Đang làm',
    icon: Clock,
    colorClass: 'text-blue-600 dark:text-blue-400',
    badgeBg: 'bg-blue-100 dark:bg-blue-900/60 text-blue-700 dark:text-blue-300',
  },
  {
    id: 'done',
    title: 'Hoàn thành',
    icon: CheckCircle2,
    colorClass: 'text-emerald-600 dark:text-emerald-400',
    badgeBg: 'bg-emerald-100 dark:bg-emerald-900/60 text-emerald-700 dark:text-emerald-300',
  },
];

const kanbanData = computed(() => store.kanbanColumns);

// Drag & Drop State
const draggedTaskId = ref<string | null>(null);
const dragOverColumn = ref<TaskStatus | null>(null);

function onDragStart(e: DragEvent, task: Task) {
  draggedTaskId.value = task.id;
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/plain', task.id);
  }
}

function onDragEnd() {
  draggedTaskId.value = null;
  dragOverColumn.value = null;
}

function onDragOver(e: DragEvent, status: TaskStatus) {
  e.preventDefault();
  if (e.dataTransfer) {
    e.dataTransfer.dropEffect = 'move';
  }
  dragOverColumn.value = status;
}

function onDragLeave(_e: DragEvent, status: TaskStatus) {
  if (dragOverColumn.value === status) {
    dragOverColumn.value = null;
  }
}

async function onDrop(e: DragEvent, targetStatus: TaskStatus) {
  e.preventDefault();
  dragOverColumn.value = null;
  const taskId = e.dataTransfer?.getData('text/plain') || draggedTaskId.value;
  if (taskId) {
    await store.moveTaskStatus(taskId, targetStatus);
  }
  draggedTaskId.value = null;
}

function getPriorityBar(priority: Priority): string {
  switch (priority) {
    case 3:
      return 'border-l-4 border-l-red-500';
    case 2:
      return 'border-l-4 border-l-amber-500';
    case 1:
      return 'border-l-4 border-l-blue-500';
    default:
      return 'border-l-4 border-l-slate-300 dark:border-l-slate-600';
  }
}

function formatTaskDate(timestamp: number | null): { text: string; isOverdue: boolean } | null {
  if (!timestamp) return null;
  const d = new Date(timestamp);
  const now = new Date();

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
  <div class="flex-1 grid grid-cols-3 gap-4 pb-12 overflow-hidden h-full select-none">
    <div
      v-for="col in columns"
      :key="col.id"
      class="flex flex-col h-full bg-slate-100/60 dark:bg-slate-900/40 rounded-xl border border-slate-200/80 dark:border-slate-800 p-3 transition-colors duration-150"
      :class="{
        'border-blue-500 ring-2 ring-blue-500/20 bg-blue-50/20 dark:bg-blue-950/20':
          dragOverColumn === col.id
      }"
      @dragover="onDragOver($event, col.id)"
      @dragleave="onDragLeave($event, col.id)"
      @drop="onDrop($event, col.id)"
    >
      <!-- Column Header -->
      <div class="flex items-center justify-between pb-3 mb-2 border-b border-slate-200/60 dark:border-slate-800/80">
        <div class="flex items-center gap-2">
          <component :is="col.icon" class="w-4 h-4" :class="col.colorClass" />
          <h3 class="text-xs font-bold uppercase tracking-wider text-slate-700 dark:text-slate-300">
            {{ col.title }}
          </h3>
        </div>
        <span class="px-2 py-0.5 text-xs font-semibold rounded-full" :class="col.badgeBg">
          {{ kanbanData[col.id].length }}
        </span>
      </div>

      <!-- Column Card List -->
      <div class="flex-1 overflow-y-auto space-y-2.5 pr-1">
        <div
          v-for="task in kanbanData[col.id]"
          :key="task.id"
          draggable="true"
          @dragstart="onDragStart($event, task)"
          @dragend="onDragEnd"
          @click="store.selectTask(task.id)"
          :class="[
            'group bg-white dark:bg-slate-800 rounded-lg p-3 shadow-sm border border-slate-200/80 dark:border-slate-700/60 hover:shadow-md cursor-grab active:cursor-grabbing transition-all relative',
            getPriorityBar(task.priority),
            store.selectedTaskId === task.id
              ? 'ring-2 ring-blue-500 bg-blue-50/30 dark:bg-blue-950/30'
              : '',
            draggedTaskId === task.id ? 'opacity-40 scale-95' : ''
          ]"
        >
          <div class="flex items-start gap-2.5">
            <!-- Round Checkbox -->
            <button
              type="button"
              @click.stop="store.toggleTaskStatus(task.id)"
              :class="[
                'w-4 h-4 rounded-full border-2 flex items-center justify-center mt-0.5 flex-shrink-0 transition-colors',
                task.status === 'done'
                  ? 'bg-emerald-500 border-emerald-500 text-white'
                  : 'border-slate-400 hover:border-blue-500'
              ]"
              title="Đổi trạng thái"
            >
              <Check v-if="task.status === 'done'" class="w-2.5 h-2.5 stroke-[3]" />
            </button>

            <!-- Card Content -->
            <div class="flex-1 min-w-0">
              <h4
                class="text-xs font-medium text-slate-800 dark:text-slate-100 line-clamp-2 leading-relaxed"
                :class="task.status === 'done' ? 'line-through text-slate-400 dark:text-slate-500' : ''"
              >
                {{ task.title }}
              </h4>

              <!-- Card Meta Badges -->
              <div class="flex items-center gap-2 mt-2 flex-wrap">
                <div
                  v-if="formatTaskDate(task.due_date)"
                  class="flex items-center gap-1 text-[11px] px-1.5 py-0.5 rounded font-medium"
                  :class="
                    formatTaskDate(task.due_date)?.isOverdue
                      ? 'text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-950/40'
                      : 'text-slate-500 dark:text-slate-400 bg-slate-100 dark:bg-slate-700/50'
                  "
                >
                  <Calendar class="w-3 h-3" />
                  <span>{{ formatTaskDate(task.due_date)?.text }}</span>
                </div>

                <div
                  v-if="task.description"
                  class="text-slate-400 dark:text-slate-500"
                  title="Có ghi chú chi tiết"
                >
                  <FileText class="w-3 h-3" />
                </div>
              </div>
            </div>

            <!-- Drag Grip Handle -->
            <div class="text-slate-300 dark:text-slate-600 opacity-0 group-hover:opacity-100 transition-opacity">
              <GripVertical class="w-3.5 h-3.5" />
            </div>
          </div>
        </div>

        <!-- Empty state placeholder for column -->
        <div
          v-if="kanbanData[col.id].length === 0"
          class="h-28 border-2 border-dashed border-slate-200 dark:border-slate-800 rounded-lg flex items-center justify-center text-xs text-slate-400 dark:text-slate-600"
        >
          Kéo thả task vào đây
        </div>
      </div>
    </div>
  </div>
</template>
