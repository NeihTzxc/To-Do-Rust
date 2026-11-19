<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import { useTaskStore } from '@/stores/taskStore';
import type { Priority, TaskStatus } from '@/types/task';
import {
  X,
  Trash2,
  Calendar,
  Check,
  Flag,
  Clock,
  Layers,
  Database,
} from 'lucide-vue-next';

const store = useTaskStore();

const currentTask = computed(() => store.selectedTask);

const title = ref('');
const description = ref('');
const status = ref<TaskStatus>('todo');
const priority = ref<Priority>(2);
const dueDate = ref<number | null>(null);

// Watch for selected task changes to update local reactive form state
watch(
  () => currentTask.value,
  (task) => {
    if (task) {
      title.value = task.title;
      description.value = task.description || '';
      status.value = task.status;
      priority.value = task.priority;
      dueDate.value = task.due_date;
    }
  },
  { immediate: true }
);

// Auto-save debounce timer
let debounceTimer: any = null;

function triggerAutoSave() {
  if (!currentTask.value) return;

  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }

  debounceTimer = setTimeout(async () => {
    if (!currentTask.value) return;
    await store.updateTask({
      id: currentTask.value.id,
      title: title.value.trim() || currentTask.value.title,
      description: description.value.trim() ? description.value.trim() : null,
      status: status.value,
      priority: priority.value,
      due_date: dueDate.value,
    });
  }, 400);
}

function handleBlurSave() {
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }
  if (!currentTask.value) return;

  store.updateTask({
    id: currentTask.value.id,
    title: title.value.trim() || currentTask.value.title,
    description: description.value.trim() ? description.value.trim() : null,
    status: status.value,
    priority: priority.value,
    due_date: dueDate.value,
  });
}

async function handleToggleStatus() {
  if (!currentTask.value) return;
  await store.toggleTaskStatus(currentTask.value.id);
  if (currentTask.value) {
    status.value = currentTask.value.status;
  }
}

async function handleDelete() {
  if (!currentTask.value) return;
  if (confirm(`Bạn có chắc muốn xóa công việc "${currentTask.value.title}"?`)) {
    await store.deleteTask(currentTask.value.id);
  }
}

function onDateChange(e: Event) {
  const val = (e.target as HTMLInputElement).value;
  if (val) {
    const parts = val.split('-');
    const d = new Date(parseInt(parts[0]), parseInt(parts[1]) - 1, parseInt(parts[2]), 23, 59, 59);
    dueDate.value = d.getTime();
  } else {
    dueDate.value = null;
  }
  triggerAutoSave();
}

const formattedInputDate = computed(() => {
  if (!dueDate.value) return '';
  const d = new Date(dueDate.value);
  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
});

function formatDateTime(timestamp: number): string {
  const d = new Date(timestamp);
  return d.toLocaleString('vi-VN', {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}
</script>

<template>
  <aside
    v-if="currentTask"
    class="w-[330px] flex-shrink-0 flex flex-col h-full bg-white dark:bg-slate-900 border-l border-slate-200 dark:border-slate-800 shadow-lg select-none transition-all z-20"
  >
    <!-- Top Action Bar -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-slate-200/80 dark:border-slate-800">
      <div class="flex items-center gap-2">
        <!-- Quick completion checkbox -->
        <button
          type="button"
          @click="handleToggleStatus"
          :class="[
            'w-5 h-5 rounded-full border-2 flex items-center justify-center transition-colors',
            currentTask.status === 'done'
              ? 'bg-emerald-500 border-emerald-500 text-white'
              : 'border-slate-400 hover:border-blue-500 text-transparent'
          ]"
          :title="currentTask.status === 'done' ? 'Đã hoàn thành' : 'Đánh dấu hoàn thành'"
        >
          <Check class="w-3 h-3 stroke-[3]" />
        </button>
        <span class="text-xs font-semibold text-slate-500 dark:text-slate-400">
          {{ currentTask.status === 'done' ? 'Đã hoàn thành' : 'Chi tiết công việc' }}
        </span>
      </div>

      <div class="flex items-center gap-1">
        <button
          @click="handleDelete"
          class="p-1.5 rounded-md text-slate-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-950/30 transition-colors"
          title="Xóa công việc"
        >
          <Trash2 class="w-4 h-4" />
        </button>
        <button
          @click="store.selectTask(null)"
          class="p-1.5 rounded-md text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
          title="Đóng chi tiết (Esc)"
        >
          <X class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Main Content Form -->
    <div class="flex-1 overflow-y-auto px-4 py-4 space-y-5">
      <!-- Title Input (Large, borderless) -->
      <div>
        <textarea
          v-model="title"
          @input="triggerAutoSave"
          @blur="handleBlurSave"
          rows="2"
          placeholder="Tên công việc..."
          class="w-full text-base font-semibold text-slate-900 dark:text-slate-100 bg-transparent border-none outline-none resize-none placeholder-slate-400 leading-snug"
        />
      </div>

      <!-- Properties Grid -->
      <div class="space-y-3 pt-1 border-t border-slate-100 dark:border-slate-800">
        <!-- Status Property -->
        <div class="flex items-center justify-between text-xs">
          <div class="flex items-center gap-2 text-slate-500 dark:text-slate-400">
            <Layers class="w-3.5 h-3.5" />
            <span>Trạng thái</span>
          </div>
          <select
            v-model="status"
            @change="triggerAutoSave"
            class="bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-md px-2 py-1 text-slate-700 dark:text-slate-200 outline-none text-xs focus:border-blue-500"
          >
            <option value="todo">Cần làm (Todo)</option>
            <option value="in_progress">Đang làm (In Progress)</option>
            <option value="done">Hoàn thành (Done)</option>
          </select>
        </div>

        <!-- Priority Property -->
        <div class="flex items-center justify-between text-xs">
          <div class="flex items-center gap-2 text-slate-500 dark:text-slate-400">
            <Flag class="w-3.5 h-3.5" />
            <span>Độ ưu tiên</span>
          </div>
          <div class="flex items-center gap-1">
            <button
              v-for="p in [
                { val: 1 as Priority, label: 'Thấp', color: 'text-blue-500 hover:bg-blue-500/10' },
                { val: 2 as Priority, label: 'Vừa', color: 'text-amber-500 hover:bg-amber-500/10' },
                { val: 3 as Priority, label: 'Cao', color: 'text-red-500 hover:bg-red-500/10' },
              ]"
              :key="p.val"
              type="button"
              @click="priority = p.val; triggerAutoSave()"
              :class="[
                'px-2 py-1 rounded text-xs font-medium transition-colors flex items-center gap-1',
                priority === p.val
                  ? 'bg-slate-200 dark:bg-slate-700 font-semibold'
                  : 'text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800'
              ]"
            >
              <Flag :class="['w-3 h-3 fill-current', p.color]" />
              <span>{{ p.label }}</span>
            </button>
          </div>
        </div>

        <!-- Due Date Property -->
        <div class="flex items-center justify-between text-xs">
          <div class="flex items-center gap-2 text-slate-500 dark:text-slate-400">
            <Calendar class="w-3.5 h-3.5" />
            <span>Ngày đến hạn</span>
          </div>
          <div class="flex items-center gap-1">
            <input
              type="date"
              :value="formattedInputDate"
              @change="onDateChange"
              class="bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-md px-2 py-1 text-slate-700 dark:text-slate-200 outline-none text-xs focus:border-blue-500"
            />
            <button
              v-if="dueDate"
              type="button"
              @click="dueDate = null; triggerAutoSave()"
              class="text-slate-400 hover:text-red-500 p-1"
              title="Xóa hạn chót"
            >
              <X class="w-3 h-3" />
            </button>
          </div>
        </div>
      </div>

      <!-- Description / Notes Section -->
      <div class="space-y-1.5 pt-2 border-t border-slate-100 dark:border-slate-800">
        <label class="block text-xs font-semibold text-slate-400 uppercase tracking-wider">
          Ghi chú chi tiết
        </label>
        <textarea
          v-model="description"
          @input="triggerAutoSave"
          @blur="handleBlurSave"
          rows="6"
          placeholder="Thêm mô tả, danh sách việc nhỏ, hoặc ghi chú công việc..."
          class="w-full text-xs text-slate-700 dark:text-slate-200 bg-slate-50 dark:bg-slate-800/60 border border-slate-200 dark:border-slate-700/80 rounded-lg p-2.5 outline-none focus:border-blue-500 dark:focus:border-blue-500 focus:ring-1 focus:ring-blue-500 placeholder-slate-400 leading-relaxed transition-all"
        />
      </div>
    </div>

    <!-- Metadata Footer -->
    <div class="px-4 py-3 bg-slate-50/70 dark:bg-slate-950/60 border-t border-slate-200/80 dark:border-slate-800 text-[11px] text-slate-400 dark:text-slate-500 space-y-1">
      <div class="flex items-center justify-between">
        <span class="flex items-center gap-1">
          <Clock class="w-3 h-3" /> Tạo lúc:
        </span>
        <span>{{ formatDateTime(currentTask.created_at) }}</span>
      </div>
      <div class="flex items-center justify-between">
        <span class="flex items-center gap-1">
          <Clock class="w-3 h-3" /> Cập nhật lúc:
        </span>
        <span>{{ formatDateTime(currentTask.updated_at) }}</span>
      </div>
      <div class="flex items-center justify-between pt-1 border-t border-slate-200/40 dark:border-slate-800/60">
        <span class="flex items-center gap-1">
          <Database class="w-3 h-3 text-blue-500" /> Trạng thái sync:
        </span>
        <span
          class="font-medium"
          :class="currentTask.sync_status === 'pending' ? 'text-amber-500' : 'text-emerald-500'"
        >
          {{ currentTask.sync_status === 'pending' ? 'Chờ đồng bộ' : 'Đã đồng bộ' }}
        </span>
      </div>
    </div>
  </aside>
</template>
