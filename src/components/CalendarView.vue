<script setup lang="ts">
import { ref, computed } from 'vue';
import { useTaskStore } from '@/stores/taskStore';
import type { CalendarViewMode, Priority } from '@/types/task';
import {
  localDateKey,
  parseDateKey,
  startOfDayMs,
  endOfDayMs,
} from '@/utils/calendarTask';
import UnscheduledDrawer from '@/components/UnscheduledDrawer.vue';
import CalendarTimeGrid from '@/components/CalendarTimeGrid.vue';
import CalendarMonthGrid from '@/components/CalendarMonthGrid.vue';
import RecurrenceSelector from '@/components/RecurrenceSelector.vue';
import {
  ChevronLeft,
  ChevronRight,
  Plus,
  PanelLeft,
  PanelLeftClose,
  Calendar,
  X,
  Clock,
  Flag,
} from 'lucide-vue-next';

const store = useTaskStore();

// Quick Create Modal State
const isCreateModalOpen = ref(false);
const newTitle = ref('');
const newDescription = ref('');
const newPriority = ref<Priority>(2);
const newStartTime = ref<number | null>(null);
const newEndTime = ref<number | null>(null);
const newIsAllDay = ref(false);
const newRecurrenceRule = ref<string | null>(null);
const newScheduleMode = ref<'time_block' | 'all_day' | 'date_range'>('time_block');

// Header Month & Year Title (e.g. "Tháng 11 2026" or "November 2026")
const currentHeaderTitle = computed(() => {
  const d = new Date(store.calendarCurrentDate);
  const monthNames = [
    'January', 'February', 'March', 'April', 'May', 'June',
    'July', 'August', 'September', 'October', 'November', 'December'
  ];
  return `${monthNames[d.getMonth()]} ${d.getFullYear()}`;
});

function openCreateModal(defaultTimes?: { startTime: number; endTime: number; dateStr?: string }) {
  if (defaultTimes) {
    newStartTime.value = defaultTimes.startTime;
    newEndTime.value = defaultTimes.endTime;
  } else {
    // Default 1 hour from current date at 09:00
    const d = new Date(store.calendarCurrentDate);
    d.setHours(9, 0, 0, 0);
    newStartTime.value = d.getTime();
    newEndTime.value = d.getTime() + 60 * 60 * 1000;
  }
  newTitle.value = '';
  newDescription.value = '';
  newPriority.value = 2;
  newIsAllDay.value = false;
  newRecurrenceRule.value = null;
  newScheduleMode.value = 'time_block';
  isCreateModalOpen.value = true;
}

function setModalScheduleMode(mode: 'time_block' | 'all_day' | 'date_range') {
  newScheduleMode.value = mode;
  const base = newStartTime.value ? new Date(newStartTime.value) : new Date(store.calendarCurrentDate);
  if (mode === 'time_block') {
    newIsAllDay.value = false;
    const s = new Date(base.getFullYear(), base.getMonth(), base.getDate(), 9, 0, 0, 0);
    newStartTime.value = s.getTime();
    newEndTime.value = s.getTime() + 60 * 60 * 1000;
  } else if (mode === 'all_day') {
    newIsAllDay.value = true;
    newStartTime.value = startOfDayMs(base);
    newEndTime.value = endOfDayMs(base);
  } else {
    newIsAllDay.value = true;
    const e = new Date(base);
    e.setDate(e.getDate() + 1);
    newStartTime.value = startOfDayMs(base);
    newEndTime.value = endOfDayMs(e);
  }
}

function onModalRangeStartChange(e: Event) {
  const val = (e.target as HTMLInputElement).value;
  if (!val) return;
  const s = parseDateKey(val);
  newStartTime.value = startOfDayMs(s);
  if (!newEndTime.value || newEndTime.value < newStartTime.value) {
    newEndTime.value = endOfDayMs(s);
  }
}

function onModalRangeEndChange(e: Event) {
  const val = (e.target as HTMLInputElement).value;
  if (!val) return;
  const end = parseDateKey(val);
  newEndTime.value = endOfDayMs(end);
  if (!newStartTime.value || newStartTime.value > newEndTime.value) {
    newStartTime.value = startOfDayMs(end);
  }
}

async function handleCreateTask() {
  if (!newTitle.value.trim()) return;

  await store.createTask({
    title: newTitle.value.trim(),
    description: newDescription.value.trim() || undefined,
    priority: newPriority.value,
    due_date: newEndTime.value,
    start_time: newStartTime.value,
    end_time: newEndTime.value,
    is_all_day: newIsAllDay.value,
    recurrence_rule: newRecurrenceRule.value,
  });

  isCreateModalOpen.value = false;
}

const formattedModalStartTime = computed(() => {
  if (!newStartTime.value) return '';
  const d = new Date(newStartTime.value);
  return `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
});

const formattedModalEndTime = computed(() => {
  if (!newEndTime.value) return '';
  const d = new Date(newEndTime.value);
  return `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
});

function onModalStartTimeChange(e: Event) {
  const val = (e.target as HTMLInputElement).value;
  if (!val || !newStartTime.value) return;
  const [h, m] = val.split(':').map(Number);
  const d = new Date(newStartTime.value);
  d.setHours(h, m, 0, 0);
  newStartTime.value = d.getTime();
  if (!newEndTime.value || newEndTime.value <= newStartTime.value) {
    newEndTime.value = newStartTime.value + 60 * 60 * 1000;
  }
}

function onModalEndTimeChange(e: Event) {
  const val = (e.target as HTMLInputElement).value;
  if (!val || !newEndTime.value) return;
  const [h, m] = val.split(':').map(Number);
  const d = new Date(newEndTime.value);
  d.setHours(h, m, 0, 0);
  newEndTime.value = d.getTime();
}
</script>

<template>
  <div class="flex-1 flex flex-col h-full min-w-0 bg-white dark:bg-slate-900 overflow-hidden relative select-none">
    <!-- 1. Calendar Header Bar -->
    <header class="flex items-center justify-between px-5 py-3 border-b border-slate-200/80 dark:border-slate-800 bg-white dark:bg-slate-900/90 z-20 flex-shrink-0">
      <!-- Left side: Toggle Unscheduled Sidebar + Month & Year Title -->
      <div class="flex items-center gap-3.5">
        <button
          type="button"
          @click="store.toggleUnscheduled"
          class="p-1.5 rounded-lg text-slate-500 hover:text-slate-800 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
          :title="store.isUnscheduledOpen ? 'Đóng danh sách chưa xếp lịch' : 'Mở danh sách chưa xếp lịch'"
        >
          <PanelLeftClose v-if="store.isUnscheduledOpen" class="w-4 h-4 text-blue-500" />
          <PanelLeft v-else class="w-4 h-4" />
        </button>

        <h1 class="text-xl font-bold tracking-tight text-slate-800 dark:text-slate-100">
          {{ currentHeaderTitle }}
        </h1>
      </div>

      <!-- Right side: Navigation (< Today >), Mode (Day/Week/Month) & + Button -->
      <div class="flex items-center gap-3">
        <!-- Navigation Buttons: < | Today | > -->
        <div class="flex items-center bg-slate-100 dark:bg-slate-800 p-0.5 rounded-lg border border-slate-200/60 dark:border-slate-700/60">
          <button
            type="button"
            @click="store.prevCalendarPeriod"
            class="p-1 rounded-md text-slate-600 dark:text-slate-300 hover:bg-white dark:hover:bg-slate-700 hover:shadow-xs transition-all"
            title="Kỳ trước"
          >
            <ChevronLeft class="w-4 h-4" />
          </button>

          <button
            type="button"
            @click="store.todayCalendarPeriod"
            class="px-2.5 py-0.5 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-white dark:hover:bg-slate-700 rounded-md transition-all"
            title="Quay lại hôm nay"
          >
            Hôm nay
          </button>

          <button
            type="button"
            @click="store.nextCalendarPeriod"
            class="p-1 rounded-md text-slate-600 dark:text-slate-300 hover:bg-white dark:hover:bg-slate-700 hover:shadow-xs transition-all"
            title="Kỳ sau"
          >
            <ChevronRight class="w-4 h-4" />
          </button>
        </div>

        <!-- Mode Switcher: Day | Week | Month -->
        <div class="hidden sm:flex items-center bg-slate-100 dark:bg-slate-800 p-0.5 rounded-lg border border-slate-200/60 dark:border-slate-700/60">
          <button
            v-for="m in [
              { id: 'day' as CalendarViewMode, label: 'Ngày' },
              { id: 'week' as CalendarViewMode, label: 'Tuần' },
              { id: 'month' as CalendarViewMode, label: 'Tháng' },
            ]"
            :key="m.id"
            type="button"
            @click="store.setCalendarViewMode(m.id)"
            :class="[
              'px-2.5 py-1 text-xs font-medium rounded-md transition-all',
              store.calendarViewMode === m.id
                ? 'bg-white dark:bg-slate-700 text-blue-600 dark:text-blue-400 shadow-xs font-semibold'
                : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'
            ]"
          >
            {{ m.label }}
          </button>
        </div>

        <!-- + New Task Button with pre-filled time -->
        <button
          type="button"
          @click="openCreateModal()"
          class="flex items-center gap-1 px-3 py-1.5 rounded-lg bg-[#3B82F6] hover:bg-blue-600 text-white text-xs font-semibold shadow-sm transition-all hover:shadow"
        >
          <Plus class="w-4 h-4" />
          <span>Thêm task</span>
        </button>
      </div>
    </header>

    <!-- 2. Main Calendar Workspace (Unscheduled Drawer + Active Grid) -->
    <div class="flex-1 flex min-h-0 overflow-hidden relative">
      <!-- Unscheduled Tasks Drawer -->
      <UnscheduledDrawer />

      <!-- Active Grid View -->
      <div class="flex-1 flex flex-col min-w-0 h-full overflow-hidden relative">
        <CalendarTimeGrid
          v-if="store.calendarViewMode === 'day' || store.calendarViewMode === 'week'"
          :mode="store.calendarViewMode"
          @create-at="openCreateModal"
        />

        <CalendarMonthGrid
          v-else-if="store.calendarViewMode === 'month'"
          @create-at="openCreateModal"
        />

        <div
          v-else-if="store.calendarViewMode === 'agenda'"
          class="flex-1 p-6 overflow-y-auto"
        >
          <h2 class="text-base font-bold text-slate-800 dark:text-slate-100 mb-4">
            Lịch trình Agenda sắp tới
          </h2>
          <div class="space-y-2">
            <div
              v-for="task in store.scheduledTasks"
              :key="task.id"
              @click="store.selectTask(task.id)"
              class="p-3 rounded-lg border border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-800/80 hover:shadow-sm cursor-pointer flex items-center justify-between"
            >
              <div>
                <p class="text-sm font-semibold text-slate-800 dark:text-slate-100">{{ task.title }}</p>
                <p class="text-xs text-slate-400 mt-0.5">
                  {{ new Date(task.start_time!).toLocaleString('vi-VN', { dateStyle: 'short', timeStyle: 'short' }) }}
                  -
                  {{ new Date(task.end_time!).toLocaleTimeString('vi-VN', { timeStyle: 'short' }) }}
                </p>
              </div>
              <span
                :class="[
                  'px-2 py-0.5 rounded text-xs font-semibold',
                  task.priority === 3 ? 'bg-rose-100 text-rose-700' : task.priority === 2 ? 'bg-blue-100 text-blue-700' : 'bg-teal-100 text-teal-700'
                ]"
              >
                {{ task.priority === 3 ? 'Cao' : task.priority === 2 ? 'Vừa' : 'Thấp' }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 3. Floating Bottom Pill Bar (Standard TickTick Style: #4A4A4A) -->
    <div class="absolute bottom-5 left-1/2 -translate-x-1/2 z-30 pointer-events-auto">
      <div
        class="bg-[#4A4A4A] text-white/80 rounded-full px-2 py-1.5 shadow-xl flex items-center gap-1 backdrop-blur-md border border-white/10"
      >
        <button
          v-for="tab in [
            { id: 'month' as CalendarViewMode, label: 'Tháng (Month)' },
            { id: 'week' as CalendarViewMode, label: 'Tuần (Week)' },
            { id: 'day' as CalendarViewMode, label: 'Ngày (Day)' },
            { id: 'agenda' as CalendarViewMode, label: 'Lịch trình (Agenda)' },
          ]"
          :key="tab.id"
          type="button"
          @click="store.setCalendarViewMode(tab.id)"
          :class="[
            'px-3 py-1 rounded-full text-xs font-medium transition-all',
            store.calendarViewMode === tab.id
              ? 'bg-white/25 text-white font-bold shadow-sm'
              : 'hover:text-white hover:bg-white/10 text-slate-300'
          ]"
        >
          {{ tab.label }}
        </button>
      </div>
    </div>

    <!-- 4. Quick Create Task Modal on Empty Slot Click -->
    <div
      v-if="isCreateModalOpen"
      class="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4"
      @click.self="isCreateModalOpen = false"
    >
      <div
        class="w-full max-w-md bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl shadow-2xl p-5 space-y-4"
      >
        <div class="flex items-center justify-between border-b border-slate-100 dark:border-slate-800 pb-3">
          <div class="flex items-center gap-2 text-slate-800 dark:text-slate-100 font-bold text-sm">
            <Calendar class="w-4 h-4 text-blue-500" />
            <span>Thêm công việc vào Lịch</span>
          </div>
          <button
            type="button"
            @click="isCreateModalOpen = false"
            class="text-slate-400 hover:text-slate-600 p-1"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        <form @submit.prevent="handleCreateTask" class="space-y-3.5">
          <div>
            <input
              v-model="newTitle"
              type="text"
              placeholder="Nhập tên công việc..."
              autofocus
              class="w-full px-3 py-2 text-sm bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg text-slate-800 dark:text-slate-100 outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
            />
          </div>

          <!-- Schedule Mode Tabs -->
          <div class="grid grid-cols-3 gap-1 bg-slate-100 dark:bg-slate-800 p-1 rounded-lg text-xs">
            <button
              v-for="m in [
                { id: 'time_block', label: 'Theo giờ' },
                { id: 'all_day', label: 'Cả ngày' },
                { id: 'date_range', label: 'Nhiều ngày' },
              ]"
              :key="m.id"
              type="button"
              @click="setModalScheduleMode(m.id as any)"
              :class="[
                'py-1 rounded-md font-medium transition-colors',
                newScheduleMode === m.id
                  ? 'bg-white dark:bg-slate-700 text-blue-600 dark:text-blue-400 shadow-xs'
                  : 'text-slate-500 hover:text-slate-800 dark:hover:text-slate-200'
              ]"
            >
              {{ m.label }}
            </button>
          </div>

          <!-- Time Range Input (Theo giờ) -->
          <div v-if="newScheduleMode === 'time_block'" class="flex items-center gap-2 text-xs">
            <Clock class="w-4 h-4 text-slate-400 flex-shrink-0" />
            <div class="flex items-center gap-1.5 flex-1">
              <input
                type="time"
                :value="formattedModalStartTime"
                @change="onModalStartTimeChange"
                class="flex-1 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-md px-2 py-1 text-slate-700 dark:text-slate-200 outline-none text-xs"
              />
              <span class="text-slate-400">→</span>
              <input
                type="time"
                :value="formattedModalEndTime"
                @change="onModalEndTimeChange"
                class="flex-1 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-md px-2 py-1 text-slate-700 dark:text-slate-200 outline-none text-xs"
              />
            </div>
          </div>

          <!-- All-day Date Input -->
          <div v-else-if="newScheduleMode === 'all_day'" class="flex items-center justify-between gap-2 text-xs">
            <span class="text-slate-500 dark:text-slate-400">Ngày:</span>
            <input
              type="date"
              :value="newStartTime ? localDateKey(new Date(newStartTime)) : ''"
              @change="onModalRangeStartChange"
              class="bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-md px-2 py-1 text-slate-700 dark:text-slate-200 outline-none text-xs"
            />
          </div>

          <!-- Multi-day Date Range Input -->
          <div v-else class="flex items-center gap-1.5 text-xs">
            <input
              type="date"
              :value="newStartTime ? localDateKey(new Date(newStartTime)) : ''"
              @change="onModalRangeStartChange"
              class="flex-1 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-md px-2 py-1 text-slate-700 dark:text-slate-200 outline-none text-xs"
            />
            <span class="text-slate-400">→</span>
            <input
              type="date"
              :value="newEndTime ? localDateKey(new Date(newEndTime)) : ''"
              @change="onModalRangeEndChange"
              class="flex-1 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-md px-2 py-1 text-slate-700 dark:text-slate-200 outline-none text-xs"
            />
          </div>

          <!-- Recurrence Selector -->
          <div class="pt-1">
            <RecurrenceSelector v-model="newRecurrenceRule" :compact="true" />
          </div>

          <!-- Priority Selector -->
          <div class="flex items-center justify-between text-xs pt-1">
            <span class="text-slate-500 dark:text-slate-400 flex items-center gap-1.5">
              <Flag class="w-3.5 h-3.5" /> Ưu tiên
            </span>
            <div class="flex items-center gap-1">
              <button
                v-for="p in [
                  { val: 1 as Priority, label: 'Thấp', color: 'text-teal-500' },
                  { val: 2 as Priority, label: 'Vừa', color: 'text-blue-500' },
                  { val: 3 as Priority, label: 'Cao', color: 'text-rose-500' },
                ]"
                :key="p.val"
                type="button"
                @click="newPriority = p.val"
                :class="[
                  'px-2 py-1 rounded text-xs font-medium transition-colors',
                  newPriority === p.val
                    ? 'bg-slate-200 dark:bg-slate-700 font-semibold text-slate-900 dark:text-white'
                    : 'text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800'
                ]"
              >
                {{ p.label }}
              </button>
            </div>
          </div>

          <div class="flex items-center justify-end gap-2 pt-3 border-t border-slate-100 dark:border-slate-800">
            <button
              type="button"
              @click="isCreateModalOpen = false"
              class="px-3 py-1.5 text-xs text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-lg transition-colors"
            >
              Hủy
            </button>
            <button
              type="submit"
              class="px-4 py-1.5 text-xs font-semibold bg-[#3B82F6] hover:bg-blue-600 text-white rounded-lg shadow-sm transition-all"
            >
              Thêm vào lịch
            </button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>

