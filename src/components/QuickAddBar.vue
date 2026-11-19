<script setup lang="ts">
import { ref, computed } from 'vue';
import { useTaskStore } from '@/stores/taskStore';
import type { Priority } from '@/types/task';
import {
  Plus,
  Flag,
  Calendar,
  X,
  CornerDownLeft,
} from 'lucide-vue-next';

const store = useTaskStore();

const title = ref('');
const priority = ref<Priority>(2); // Mặc định Medium
const dueDate = ref<number | null>(null);

const showPriorityMenu = ref(false);
const showDateMenu = ref(false);

const priorityList: { value: Priority; label: string; color: string; bg: string }[] = [
  { value: 3, label: 'Ưu tiên Cao', color: 'text-red-500', bg: 'hover:bg-red-500/10' },
  { value: 2, label: 'Ưu tiên Vừa', color: 'text-amber-500', bg: 'hover:bg-amber-500/10' },
  { value: 1, label: 'Ưu tiên Thấp', color: 'text-blue-500', bg: 'hover:bg-blue-500/10' },
];

const currentPriorityColor = computed(() => {
  switch (priority.value) {
    case 3:
      return 'text-red-500';
    case 2:
      return 'text-amber-500';
    case 1:
      return 'text-blue-500';
    default:
      return 'text-slate-400';
  }
});

const formattedDueDate = computed(() => {
  if (!dueDate.value) return null;
  const d = new Date(dueDate.value);
  const now = new Date();
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

  if (isSameDay) return 'Hôm nay';
  if (isTomorrow) return 'Ngày mai';
  return `${d.getDate()}/${d.getMonth() + 1}`;
});

function setToday() {
  const d = new Date();
  d.setHours(23, 59, 59, 999);
  dueDate.value = d.getTime();
  showDateMenu.value = false;
}

function setTomorrow() {
  const d = new Date();
  d.setDate(d.getDate() + 1);
  d.setHours(23, 59, 59, 999);
  dueDate.value = d.getTime();
  showDateMenu.value = false;
}

function onCustomDateChange(e: Event) {
  const val = (e.target as HTMLInputElement).value;
  if (val) {
    const parts = val.split('-');
    const d = new Date(parseInt(parts[0]), parseInt(parts[1]) - 1, parseInt(parts[2]), 23, 59, 59);
    dueDate.value = d.getTime();
  } else {
    dueDate.value = null;
  }
  showDateMenu.value = false;
}

async function handleCreate() {
  if (!title.value.trim()) return;

  const text = title.value.trim();
  const prio = priority.value;
  const due = dueDate.value;

  // Reset form
  title.value = '';
  dueDate.value = null;
  priority.value = 2;
  showPriorityMenu.value = false;
  showDateMenu.value = false;

  await store.createTask({
    title: text,
    priority: prio,
    due_date: due,
  });
}
</script>

<template>
  <div class="relative w-full mb-4">
    <div
      class="flex items-center gap-2 bg-white dark:bg-slate-800/90 border border-slate-200 dark:border-slate-700/80 rounded-xl px-3.5 py-2.5 shadow-sm hover:border-slate-300 dark:hover:border-slate-600 focus-within:border-blue-500 dark:focus-within:border-blue-500 focus-within:ring-2 focus-within:ring-blue-500/20 transition-all"
    >
      <div class="text-slate-400 dark:text-slate-500 flex-shrink-0">
        <Plus class="w-5 h-5 text-blue-500" />
      </div>

      <!-- Main Input -->
      <input
        v-model="title"
        @keydown.enter.prevent="handleCreate"
        type="text"
        placeholder="Thêm công việc mới... Nhấn Enter để lưu"
        class="flex-1 bg-transparent border-none outline-none text-sm text-slate-800 dark:text-slate-100 placeholder-slate-400 dark:placeholder-slate-500"
      />

      <!-- Action Buttons -->
      <div class="flex items-center gap-1.5 flex-shrink-0">
        <!-- Date Selector Badge or Button -->
        <div class="relative">
          <button
            v-if="!formattedDueDate"
            type="button"
            @click="showDateMenu = !showDateMenu; showPriorityMenu = false"
            class="flex items-center gap-1 px-2 py-1 rounded-md text-xs font-medium text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-700/60 transition-colors"
            title="Đặt ngày đến hạn"
          >
            <Calendar class="w-4 h-4" />
          </button>
          <div
            v-else
            class="flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-blue-50 dark:bg-blue-900/40 text-blue-600 dark:text-blue-300 border border-blue-200 dark:border-blue-800"
          >
            <Calendar class="w-3.5 h-3.5" />
            <span>{{ formattedDueDate }}</span>
            <button
              @click="dueDate = null"
              class="hover:text-red-500 ml-0.5"
              title="Xóa ngày hạn"
            >
              <X class="w-3 h-3" />
            </button>
          </div>

          <!-- Date Dropdown Popover -->
          <div
            v-if="showDateMenu"
            class="absolute right-0 top-full mt-2 w-48 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg shadow-xl py-1 z-30"
          >
            <div class="px-2 py-1 text-[11px] font-semibold text-slate-400 uppercase tracking-wider">
              Chọn ngày hạn
            </div>
            <button
              @click="setToday"
              class="w-full text-left px-3 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-700 flex items-center justify-between"
            >
              <span>Hôm nay</span>
              <span class="text-[10px] text-slate-400">Tối nay 23:59</span>
            </button>
            <button
              @click="setTomorrow"
              class="w-full text-left px-3 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-700 flex items-center justify-between"
            >
              <span>Ngày mai</span>
            </button>
            <div class="border-t border-slate-100 dark:border-slate-700 my-1"></div>
            <div class="px-3 py-1.5">
              <label class="block text-[11px] text-slate-400 mb-1">Tùy chọn ngày:</label>
              <input
                type="date"
                @change="onCustomDateChange"
                class="w-full text-xs bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded px-2 py-1 text-slate-700 dark:text-slate-200 outline-none focus:border-blue-500"
              />
            </div>
          </div>
        </div>

        <!-- Priority Selector Button -->
        <div class="relative">
          <button
            type="button"
            @click="showPriorityMenu = !showPriorityMenu; showDateMenu = false"
            class="p-1 rounded-md hover:bg-slate-100 dark:hover:bg-slate-700/60 transition-colors"
            :class="currentPriorityColor"
            title="Độ ưu tiên"
          >
            <Flag class="w-4 h-4 fill-current" />
          </button>

          <!-- Priority Dropdown Popover -->
          <div
            v-if="showPriorityMenu"
            class="absolute right-0 top-full mt-2 w-36 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg shadow-xl py-1 z-30"
          >
            <div class="px-2 py-1 text-[11px] font-semibold text-slate-400 uppercase tracking-wider">
              Độ ưu tiên
            </div>
            <button
              v-for="p in priorityList"
              :key="p.value"
              @click="priority = p.value; showPriorityMenu = false"
              :class="[
                'w-full text-left px-3 py-1.5 text-xs flex items-center gap-2 transition-colors',
                p.bg,
                priority === p.value ? 'bg-slate-100 dark:bg-slate-700 font-semibold' : ''
              ]"
            >
              <Flag :class="['w-3.5 h-3.5 fill-current', p.color]" />
              <span class="text-slate-700 dark:text-slate-200">{{ p.label }}</span>
            </button>
          </div>
        </div>

        <!-- Submit Button -->
        <button
          type="button"
          @click="handleCreate"
          :disabled="!title.trim()"
          class="flex items-center justify-center w-7 h-7 rounded-lg transition-all"
          :class="
            title.trim()
              ? 'bg-blue-600 text-white hover:bg-blue-700 shadow-sm cursor-pointer'
              : 'text-slate-300 dark:text-slate-600 cursor-not-allowed'
          "
          title="Tạo task (Enter)"
        >
          <CornerDownLeft class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  </div>
</template>
