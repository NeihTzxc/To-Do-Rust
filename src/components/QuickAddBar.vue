<script setup lang="ts">
import { ref, computed } from 'vue';
import { useTaskStore } from '@/stores/taskStore';
import type { Priority } from '@/types/task';
import {
  localDateKey,
  parseDateKey,
  startOfDayMs,
  endOfDayMs,
  formatRecurrenceLabel,
} from '@/utils/calendarTask';
import RecurrenceSelector from '@/components/RecurrenceSelector.vue';
import {
  Plus,
  Flag,
  Calendar,
  Repeat,
  X,
  CornerDownLeft,
} from 'lucide-vue-next';

const store = useTaskStore();

type QuickScheduleMode = 'none' | 'time_block' | 'all_day' | 'date_range';

const title = ref('');
const priority = ref<Priority>(2); // Mặc định Medium
const dueDate = ref<number | null>(null);
const startTime = ref<number | null>(null);
const endTime = ref<number | null>(null);
const isAllDay = ref(false);
const recurrenceRule = ref<string | null>(null);
const scheduleMode = ref<QuickScheduleMode>('none');

const showPriorityMenu = ref(false);
const showDateMenu = ref(false);
const showRecurrenceMenu = ref(false);

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

const formattedScheduleBadge = computed(() => {
  if (scheduleMode.value === 'date_range' && startTime.value && endTime.value) {
    const s = new Date(startTime.value);
    const e = new Date(endTime.value);
    return `${s.getDate()}/${s.getMonth() + 1} → ${e.getDate()}/${e.getMonth() + 1}`;
  }
  if (scheduleMode.value === 'time_block' && startTime.value && endTime.value) {
    const s = new Date(startTime.value);
    const e = new Date(endTime.value);
    const tStr = `${String(s.getHours()).padStart(2, '0')}:${String(s.getMinutes()).padStart(2, '0')}-${String(e.getHours()).padStart(2, '0')}:${String(e.getMinutes()).padStart(2, '0')}`;
    return `${s.getDate()}/${s.getMonth() + 1} ${tStr}`;
  }
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

  const dayLabel = isSameDay ? 'Hôm nay' : isTomorrow ? 'Ngày mai' : `${d.getDate()}/${d.getMonth() + 1}`;
  return isAllDay.value ? `${dayLabel} (Cả ngày)` : dayLabel;
});

const recurrenceLabel = computed(() => formatRecurrenceLabel(recurrenceRule.value));

function clearSchedule() {
  dueDate.value = null;
  startTime.value = null;
  endTime.value = null;
  isAllDay.value = false;
  scheduleMode.value = 'none';
}

function setToday() {
  const d = new Date();
  scheduleMode.value = 'all_day';
  isAllDay.value = true;
  startTime.value = startOfDayMs(d);
  endTime.value = endOfDayMs(d);
  dueDate.value = endTime.value;
  showDateMenu.value = false;
}

function setTomorrow() {
  const d = new Date();
  d.setDate(d.getDate() + 1);
  scheduleMode.value = 'all_day';
  isAllDay.value = true;
  startTime.value = startOfDayMs(d);
  endTime.value = endOfDayMs(d);
  dueDate.value = endTime.value;
  showDateMenu.value = false;
}

function selectMode(mode: QuickScheduleMode) {
  scheduleMode.value = mode;
  const base = dueDate.value ? new Date(dueDate.value) : new Date();
  if (mode === 'none') {
    clearSchedule();
    showDateMenu.value = false;
  } else if (mode === 'all_day') {
    isAllDay.value = true;
    startTime.value = startOfDayMs(base);
    endTime.value = endOfDayMs(base);
    dueDate.value = endTime.value;
  } else if (mode === 'time_block') {
    isAllDay.value = false;
    const s = new Date(base.getFullYear(), base.getMonth(), base.getDate(), 9, 0, 0, 0);
    const e = new Date(base.getFullYear(), base.getMonth(), base.getDate(), 10, 0, 0, 0);
    startTime.value = s.getTime();
    endTime.value = e.getTime();
    dueDate.value = endOfDayMs(base);
  } else if (mode === 'date_range') {
    isAllDay.value = true;
    const e = new Date(base);
    e.setDate(e.getDate() + 1);
    startTime.value = startOfDayMs(base);
    endTime.value = endOfDayMs(e);
    dueDate.value = endTime.value;
  }
}

function onCustomDateChange(e: Event) {
  const val = (e.target as HTMLInputElement).value;
  if (!val) {
    clearSchedule();
    return;
  }
  const d = parseDateKey(val);
  if (scheduleMode.value === 'time_block' && startTime.value && endTime.value) {
    const oldS = new Date(startTime.value);
    const dur = Math.max(30 * 60 * 1000, endTime.value - startTime.value);
    const newS = new Date(d.getFullYear(), d.getMonth(), d.getDate(), oldS.getHours(), oldS.getMinutes(), 0, 0);
    startTime.value = newS.getTime();
    endTime.value = startTime.value + dur;
    dueDate.value = endOfDayMs(d);
  } else {
    scheduleMode.value = 'all_day';
    isAllDay.value = true;
    startTime.value = startOfDayMs(d);
    endTime.value = endOfDayMs(d);
    dueDate.value = endTime.value;
  }
}

function onQuickTimeStartChange(e: Event) {
  const val = (e.target as HTMLInputElement).value;
  if (!val || !startTime.value) return;
  const [h, m] = val.split(':').map(Number);
  const d = new Date(startTime.value);
  d.setHours(h, m, 0, 0);
  startTime.value = d.getTime();
  if (!endTime.value || endTime.value <= startTime.value) {
    endTime.value = startTime.value + 60 * 60 * 1000;
  }
}

function onQuickTimeEndChange(e: Event) {
  const val = (e.target as HTMLInputElement).value;
  if (!val || !endTime.value) return;
  const [h, m] = val.split(':').map(Number);
  const d = new Date(endTime.value);
  d.setHours(h, m, 0, 0);
  endTime.value = d.getTime();
}

function onQuickRangeStartChange(e: Event) {
  const val = (e.target as HTMLInputElement).value;
  if (!val) return;
  const s = parseDateKey(val);
  startTime.value = startOfDayMs(s);
  if (!endTime.value || endTime.value < startTime.value) {
    endTime.value = endOfDayMs(s);
  }
  dueDate.value = endTime.value;
}

function onQuickRangeEndChange(e: Event) {
  const val = (e.target as HTMLInputElement).value;
  if (!val) return;
  const end = parseDateKey(val);
  endTime.value = endOfDayMs(end);
  if (!startTime.value || startTime.value > endTime.value) {
    startTime.value = startOfDayMs(end);
  }
  dueDate.value = endTime.value;
}

async function handleCreate() {
  if (!title.value.trim()) return;

  const text = title.value.trim();
  const prio = priority.value;
  const due = dueDate.value;
  const sTime = startTime.value;
  const eTime = endTime.value;
  const allDay = isAllDay.value;
  const rec = recurrenceRule.value;

  // Reset form
  title.value = '';
  clearSchedule();
  recurrenceRule.value = null;
  priority.value = 2;
  showPriorityMenu.value = false;
  showDateMenu.value = false;
  showRecurrenceMenu.value = false;

  await store.createTask({
    title: text,
    priority: prio,
    due_date: due,
    start_time: sTime,
    end_time: eTime,
    is_all_day: allDay,
    recurrence_rule: rec,
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
        <!-- Date & Schedule Mode Selector -->
        <div class="relative">
          <button
            v-if="!formattedScheduleBadge"
            type="button"
            @click="showDateMenu = !showDateMenu; showPriorityMenu = false; showRecurrenceMenu = false"
            class="flex items-center gap-1 px-2 py-1 rounded-md text-xs font-medium text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-700/60 transition-colors"
            title="Thiết lập thời gian & chế độ lịch"
          >
            <Calendar class="w-4 h-4" />
          </button>
          <div
            v-else
            @click="showDateMenu = !showDateMenu; showPriorityMenu = false; showRecurrenceMenu = false"
            class="flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-blue-50 dark:bg-blue-900/40 text-blue-600 dark:text-blue-300 border border-blue-200 dark:border-blue-800 cursor-pointer"
          >
            <Calendar class="w-3.5 h-3.5" />
            <span>{{ formattedScheduleBadge }}</span>
            <button
              type="button"
              @click.stop="clearSchedule"
              class="hover:text-red-500 ml-0.5"
              title="Xóa lịch"
            >
              <X class="w-3 h-3" />
            </button>
          </div>

          <!-- Date & Time Mode Dropdown Popover -->
          <div
            v-if="showDateMenu"
            class="absolute right-0 top-full mt-2 w-72 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-xl shadow-xl p-2.5 z-30 space-y-2.5"
          >
            <div class="grid grid-cols-2 gap-1.5">
              <button
                type="button"
                @click="setToday"
                class="text-left px-2.5 py-1.5 text-xs rounded-md bg-slate-50 dark:bg-slate-900/60 text-slate-700 dark:text-slate-200 hover:bg-blue-50 dark:hover:bg-blue-900/30"
              >
                Hôm nay (Cả ngày)
              </button>
              <button
                type="button"
                @click="setTomorrow"
                class="text-left px-2.5 py-1.5 text-xs rounded-md bg-slate-50 dark:bg-slate-900/60 text-slate-700 dark:text-slate-200 hover:bg-blue-50 dark:hover:bg-blue-900/30"
              >
                Ngày mai (Cả ngày)
              </button>
            </div>

            <!-- Schedule Mode Tabs -->
            <div class="grid grid-cols-3 gap-1 bg-slate-100 dark:bg-slate-900 p-1 rounded-lg text-[11px]">
              <button
                v-for="m in [
                  { id: 'all_day', label: 'Cả ngày' },
                  { id: 'time_block', label: 'Theo giờ' },
                  { id: 'date_range', label: 'Nhiều ngày' },
                ]"
                :key="m.id"
                type="button"
                @click="selectMode(m.id as any)"
                :class="[
                  'py-1 rounded font-medium transition-colors',
                  scheduleMode === m.id
                    ? 'bg-white dark:bg-slate-700 text-blue-600 dark:text-blue-400 shadow-xs'
                    : 'text-slate-500 hover:text-slate-700 dark:hover:text-slate-200'
                ]"
              >
                {{ m.label }}
              </button>
            </div>

            <!-- Single date picker (for all_day or time_block) -->
            <div v-if="scheduleMode !== 'date_range'" class="space-y-1.5">
              <input
                type="date"
                :value="startTime ? localDateKey(new Date(startTime)) : (dueDate ? localDateKey(new Date(dueDate)) : '')"
                @change="onCustomDateChange"
                class="w-full text-xs bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-md px-2 py-1 text-slate-700 dark:text-slate-200 outline-none focus:border-blue-500"
              />
              <div v-if="scheduleMode === 'time_block' && startTime && endTime" class="flex items-center gap-1.5">
                <input
                  type="time"
                  :value="`${String(new Date(startTime).getHours()).padStart(2, '0')}:${String(new Date(startTime).getMinutes()).padStart(2, '0')}`"
                  @change="onQuickTimeStartChange"
                  class="flex-1 text-xs bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-md px-2 py-1 text-slate-700 dark:text-slate-200 outline-none"
                />
                <span class="text-xs text-slate-400">→</span>
                <input
                  type="time"
                  :value="`${String(new Date(endTime).getHours()).padStart(2, '0')}:${String(new Date(endTime).getMinutes()).padStart(2, '0')}`"
                  @change="onQuickTimeEndChange"
                  class="flex-1 text-xs bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-md px-2 py-1 text-slate-700 dark:text-slate-200 outline-none"
                />
              </div>
            </div>

            <!-- Multi-day date range picker -->
            <div v-else class="flex items-center gap-1">
              <input
                type="date"
                :value="startTime ? localDateKey(new Date(startTime)) : ''"
                @change="onQuickRangeStartChange"
                class="flex-1 min-w-0 text-[11px] bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-md px-1.5 py-1 text-slate-700 dark:text-slate-200 outline-none"
              />
              <span class="text-xs text-slate-400">→</span>
              <input
                type="date"
                :value="endTime ? localDateKey(new Date(endTime)) : ''"
                @change="onQuickRangeEndChange"
                class="flex-1 min-w-0 text-[11px] bg-slate-50 dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-md px-1.5 py-1 text-slate-700 dark:text-slate-200 outline-none"
              />
            </div>

            <div class="flex justify-end pt-1 border-t border-slate-100 dark:border-slate-700">
              <button
                type="button"
                @click="showDateMenu = false"
                class="px-2.5 py-1 text-xs font-medium bg-blue-600 text-white rounded-md hover:bg-blue-700"
              >
                Xong
              </button>
            </div>
          </div>
        </div>

        <!-- Recurrence Selector Button -->
        <div class="relative">
          <button
            type="button"
            @click="showRecurrenceMenu = !showRecurrenceMenu; showDateMenu = false; showPriorityMenu = false"
            class="flex items-center gap-1 px-1.5 py-1 rounded-md text-xs font-medium transition-colors"
            :class="recurrenceLabel ? 'bg-indigo-50 dark:bg-indigo-900/40 text-indigo-600 dark:text-indigo-300' : 'text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-700/60'"
            title="Thiết lập lặp lại"
          >
            <Repeat class="w-4 h-4" />
            <span v-if="recurrenceLabel" class="max-w-[70px] truncate">{{ recurrenceLabel }}</span>
          </button>

          <div
            v-if="showRecurrenceMenu"
            class="absolute right-0 top-full mt-2 w-64 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-xl shadow-xl p-3 z-30 space-y-2"
          >
            <div class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">
              Chu kỳ lặp lại
            </div>
            <RecurrenceSelector v-model="recurrenceRule" :compact="true" />
            <div class="flex justify-end pt-1">
              <button
                type="button"
                @click="showRecurrenceMenu = false"
                class="px-2.5 py-1 text-xs font-medium bg-blue-600 text-white rounded-md hover:bg-blue-700"
              >
                Xong
              </button>
            </div>
          </div>
        </div>

        <!-- Priority Selector Button -->
        <div class="relative">
          <button
            type="button"
            @click="showPriorityMenu = !showPriorityMenu; showDateMenu = false; showRecurrenceMenu = false"
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
