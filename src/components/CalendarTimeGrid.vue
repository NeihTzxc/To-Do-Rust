<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useTaskStore } from '@/stores/taskStore';
import type { Task, Priority } from '@/types/task';
import {
  calendarInterval,
  isAllDayOrMultiDay,
  taskDateSpan,
  localDateKey,
} from '@/utils/calendarTask';
import {
  Check,
  ChevronDown,
  ChevronUp,
  Clock,
  Plus,
  Repeat,
} from 'lucide-vue-next';

const props = defineProps<{
  mode: 'day' | 'week';
}>();

const emit = defineEmits<{
  (e: 'createAt', payload: { startTime: number; endTime: number; dateStr: string }): void;
}>();

const store = useTaskStore();

// Track real-time clock for current-time indicator line
const nowTimestamp = ref(Date.now());
let timerInterval: any = null;

onMounted(() => {
  timerInterval = setInterval(() => {
    nowTimestamp.value = Date.now();
  }, 60000); // update every minute
});

onUnmounted(() => {
  if (timerInterval) clearInterval(timerInterval);
});

// Height configuration
const HOUR_ROW_HEIGHT = 56; // 56px per hour
const MINUTE_HEIGHT = HOUR_ROW_HEIGHT / 60;

// Visible hours calculation
const hours = computed(() => {
  const allHours: number[] = [];
  const startH = store.isNightCollapsed ? 7 : 0;
  for (let h = startH; h <= 23; h++) {
    allHours.push(h);
  }
  return allHours;
});

// Calculate day columns based on view mode and store.calendarCurrentDate
const days = computed(() => {
  const current = new Date(store.calendarCurrentDate);
  const result: { date: Date; dateStr: string; dayName: string; dayNum: number; isToday: boolean }[] = [];

  const todayStr = localDateKey(new Date());

  if (props.mode === 'day') {
    const d = new Date(current);
    const dateStr = localDateKey(d);
    const dayNames = ['CN', 'Thứ 2', 'Thứ 3', 'Thứ 4', 'Thứ 5', 'Thứ 6', 'Thứ 7'];
    result.push({
      date: d,
      dateStr,
      dayName: dayNames[d.getDay()],
      dayNum: d.getDate(),
      isToday: dateStr === todayStr,
    });
  } else {
    // Week view: calculate Monday to Sunday of the current date's week
    const currentDay = current.getDay(); // 0 is Sunday, 1 is Monday...
    const mondayOffset = currentDay === 0 ? -6 : 1 - currentDay;
    const monday = new Date(current);
    monday.setDate(current.getDate() + mondayOffset);
    monday.setHours(0, 0, 0, 0);

    const dayNames = ['Thứ 2', 'Thứ 3', 'Thứ 4', 'Thứ 5', 'Thứ 6', 'Thứ 7', 'CN'];

    for (let i = 0; i < 7; i++) {
      const d = new Date(monday);
      d.setDate(monday.getDate() + i);
      const dateStr = localDateKey(d);
      result.push({
        date: d,
        dateStr,
        dayName: dayNames[i],
        dayNum: d.getDate(),
        isToday: dateStr === todayStr,
      });
    }
  }

  return result;
});

// Current time indicator position
const currentTimeIndicator = computed(() => {
  const now = new Date(nowTimestamp.value);
  const startHour = store.isNightCollapsed ? 7 : 0;
  const currentHour = now.getHours();
  const currentMin = now.getMinutes();

  if (currentHour < startHour) return null;

  const totalMinutes = (currentHour - startHour) * 60 + currentMin;
  const top = totalMinutes * MINUTE_HEIGHT;
  const timeStr = `${String(currentHour).padStart(2, '0')}:${String(currentMin).padStart(2, '0')}`;

  return { top, timeStr };
});

// Filter tasks for each day column
function getTasksForDay(date: Date): Task[] {
  const dayStart = new Date(date);
  dayStart.setHours(0, 0, 0, 0);
  const startMs = dayStart.getTime();

  const dayEnd = new Date(date);
  dayEnd.setHours(23, 59, 59, 999);
  const endMs = dayEnd.getTime();

  return store.scheduledTasks.filter((t) => {
    const interval = calendarInterval(t);
    return interval != null && interval.start <= endMs && interval.end > startMs;
  });
}

interface AllDaySegment {
  task: Task;
  startCol: number;
  spanCols: number;
  lane: number;
  continuesBefore: boolean;
  continuesAfter: boolean;
}

const allDaySegments = computed<AllDaySegment[]>(() => {
  if (days.value.length === 0) return [];
  const firstKey = days.value[0].dateStr;
  const lastKey = days.value[days.value.length - 1].dateStr;

  const matching: {
    task: Task;
    startIdx: number;
    endIdx: number;
    continuesBefore: boolean;
    continuesAfter: boolean;
  }[] = [];

  for (const task of store.tasks) {
    if (!isAllDayOrMultiDay(task)) continue;
    const span = taskDateSpan(task);
    if (!span) continue;
    if (span.endKey < firstKey || span.startKey > lastKey) continue;

    let startIdx = days.value.findIndex((d) => d.dateStr >= span.startKey);
    if (startIdx === -1) startIdx = 0;

    let endIdx = days.value.length - 1;
    for (let i = days.value.length - 1; i >= 0; i--) {
      if (days.value[i].dateStr <= span.endKey) {
        endIdx = i;
        break;
      }
    }
    if (endIdx < startIdx) continue;

    matching.push({
      task,
      startIdx,
      endIdx,
      continuesBefore: span.startKey < firstKey,
      continuesAfter: span.endKey > lastKey,
    });
  }

  // Sắp xếp: task kéo dài nhiều ngày lên trước
  matching.sort((a, b) => {
    const spanDiff = (b.endIdx - b.startIdx) - (a.endIdx - a.startIdx);
    if (spanDiff !== 0) return spanDiff;
    return a.startIdx - b.startIdx;
  });

  const laneEndCols: number[] = [];
  const segments: AllDaySegment[] = [];

  for (const item of matching) {
    let lane = laneEndCols.findIndex((endCol) => endCol < item.startIdx);
    if (lane === -1) {
      lane = laneEndCols.length;
      laneEndCols.push(item.endIdx);
    } else {
      laneEndCols[lane] = item.endIdx;
    }

    segments.push({
      task: item.task,
      startCol: item.startIdx + 1,
      spanCols: item.endIdx - item.startIdx + 1,
      lane,
      continuesBefore: item.continuesBefore,
      continuesAfter: item.continuesAfter,
    });
  }

  return segments;
});

// Compute style (top, height) for a task block on the grid
function getTaskStyle(task: Task, dayDate: Date) {
  const interval = calendarInterval(task);
  if (!interval) return {};

  const dayStart = new Date(dayDate);
  dayStart.setHours(0, 0, 0, 0);
  const dayStartMs = dayStart.getTime();

  const taskStart = new Date(Math.max(interval.start, dayStartMs));
  const dayEnd = new Date(dayStart);
  dayEnd.setDate(dayEnd.getDate() + 1);
  const taskEnd = new Date(Math.min(interval.end, dayEnd.getTime()));

  const startHour = store.isNightCollapsed ? 7 : 0;
  const taskStartMinutes = taskStart.getHours() * 60 + taskStart.getMinutes();
  const taskEndMinutes = taskEnd.getHours() * 60 + taskEnd.getMinutes();

  const adjustedStartMinutes = Math.max(0, taskStartMinutes - startHour * 60);
  const durationMinutes = Math.max(20, taskEndMinutes - taskStartMinutes);

  const top = adjustedStartMinutes * MINUTE_HEIGHT;
  const height = Math.max(26, durationMinutes * MINUTE_HEIGHT - 2);

  return {
    top: `${top}px`,
    height: `${height}px`,
  };
}

// Pastel colors by priority
function getPriorityColorClasses(p: Priority): string {
  switch (p) {
    case 3: // High priority: pastel red / peach
      return 'bg-[#FADBD8] border-[#F1948A] text-slate-800 shadow-sm';
    case 2: // Medium priority: pastel blue
      return 'bg-[#85A5FF] border-[#597EF7] text-slate-900 shadow-sm';
    case 1: // Low priority: pastel teal / cyan
      return 'bg-[#A8DCE7] border-[#72C8D8] text-slate-900 shadow-sm';
    default: // Default / None: soft pastel yellow
      return 'bg-[#E9EBB2] border-[#D4D785] text-slate-800 shadow-sm';
  }
}

function formatTaskTime(task: Task): string {
  if (!task.start_time || !task.end_time) return '';
  const s = new Date(task.start_time);
  const e = new Date(task.end_time);
  const sStr = `${String(s.getHours()).padStart(2, '0')}:${String(s.getMinutes()).padStart(2, '0')}`;
  const eStr = `${String(e.getHours()).padStart(2, '0')}:${String(e.getMinutes()).padStart(2, '0')}`;
  return `${sStr} - ${eStr}`;
}

// Drag & Drop Handling
function handleCardDragStart(e: DragEvent, task: Task) {
  if (e.dataTransfer) {
    e.dataTransfer.setData('application/json', JSON.stringify({ taskId: task.id, type: 'card' }));
    e.dataTransfer.setData('text/plain', task.id);
    e.dataTransfer.effectAllowed = 'move';
  }
}

async function handleCellDrop(e: DragEvent, day: { date: Date; dateStr: string }, hour: number) {
  e.preventDefault();
  const rawData = e.dataTransfer?.getData('application/json');
  let taskId = '';
  if (rawData) {
    try {
      const parsed = JSON.parse(rawData);
      taskId = parsed.taskId;
    } catch {
      taskId = e.dataTransfer?.getData('text/plain') || '';
    }
  } else {
    taskId = e.dataTransfer?.getData('text/plain') || '';
  }

  if (!taskId) return;

  const targetDate = new Date(day.date);
  targetDate.setHours(hour, 0, 0, 0);
  const startTime = targetDate.getTime();
  const endTime = startTime + 60 * 60 * 1000; // 1 hour block

  await store.updateTaskSchedule(taskId, startTime, endTime);
}

// Click on empty hour slot to quickly create task
function handleCellClick(day: { date: Date; dateStr: string }, hour: number) {
  const targetDate = new Date(day.date);
  targetDate.setHours(hour, 0, 0, 0);
  const startTime = targetDate.getTime();
  const endTime = startTime + 60 * 60 * 1000;

  emit('createAt', {
    startTime,
    endTime,
    dateStr: day.dateStr,
  });
}

// Resizing card duration via bottom edge handle
let resizingTask: Task | null = null;
let resizeStartY = 0;
let resizeOriginalEnd = 0;

function startResizing(e: MouseEvent, task: Task) {
  e.stopPropagation();
  resizingTask = task;
  resizeStartY = e.clientY;
  resizeOriginalEnd = task.end_time || (task.start_time! + 3600000);

  window.addEventListener('mousemove', onResizing);
  window.addEventListener('mouseup', stopResizing);
}

function onResizing(e: MouseEvent) {
  if (!resizingTask) return;
  const deltaY = e.clientY - resizeStartY;
  const deltaMinutes = Math.round(deltaY / MINUTE_HEIGHT / 15) * 15; // snap to 15 mins
  const newEnd = Math.max(resizingTask.start_time! + 15 * 60 * 1000, resizeOriginalEnd + deltaMinutes * 60 * 1000);
  resizingTask.end_time = newEnd;
}

async function stopResizing() {
  window.removeEventListener('mousemove', onResizing);
  window.removeEventListener('mouseup', stopResizing);
  if (resizingTask) {
    await store.updateTaskSchedule(resizingTask.id, resizingTask.start_time, resizingTask.end_time);
    resizingTask = null;
  }
}
</script>

<template>
  <div class="flex-1 flex flex-col h-full min-h-0 bg-white dark:bg-slate-900 overflow-hidden select-none">
    <!-- Day Headers Bar -->
    <div class="flex border-b border-slate-200/80 dark:border-slate-800 bg-slate-50/70 dark:bg-slate-900/90 flex-shrink-0 z-10">
      <!-- Left Corner (Time Axis Spacer) -->
      <div class="w-16 flex-shrink-0 border-r border-slate-200/80 dark:border-slate-800 flex items-center justify-center p-2 text-[11px] font-medium text-slate-400">
        Giờ
      </div>

      <!-- Day Columns Headers -->
      <div class="flex-1 grid" :style="{ gridTemplateColumns: `repeat(${days.length}, minmax(0, 1fr))` }">
        <div
          v-for="d in days"
          :key="d.dateStr"
          class="flex flex-col items-center justify-center py-2.5 border-r last:border-r-0 border-slate-200/80 dark:border-slate-800 text-center"
        >
          <span class="text-[11px] font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider">
            {{ d.dayName }}
          </span>
          <div
            :class="[
              'mt-1 w-7 h-7 rounded-full flex items-center justify-center text-xs font-bold transition-all',
              d.isToday
                ? 'bg-[#3B82F6] text-white shadow-sm ring-2 ring-blue-400/30'
                : 'text-slate-700 dark:text-slate-200 hover:bg-slate-200/50 dark:hover:bg-slate-800'
            ]"
          >
            {{ d.dayNum }}
          </div>
        </div>
      </div>
    </div>

    <!-- All-day & Multi-day Tasks Strip (Hỗ trợ kéo dài nhiều ngày ngang các cột) -->
    <div v-if="allDaySegments.length > 0" class="flex flex-shrink-0 border-b border-slate-200/80 dark:border-slate-800 bg-slate-50/50 dark:bg-slate-900/80">
      <div class="w-16 flex-shrink-0 border-r border-slate-200/80 dark:border-slate-800 px-1 py-2 text-center text-[10px] font-medium text-slate-500 flex items-center justify-center">
        Cả ngày
      </div>
      <div
        class="flex-1 grid gap-y-1 p-1 relative"
        :style="{ gridTemplateColumns: `repeat(${days.length}, minmax(0, 1fr))` }"
      >
        <button
          v-for="seg in allDaySegments"
          :key="seg.task.id"
          type="button"
          @click="store.selectTask(seg.task.id)"
          :style="{
            gridColumn: `${seg.startCol} / span ${seg.spanCols}`,
            gridRow: `${seg.lane + 1}`
          }"
          :class="[
            'flex items-center gap-1.5 min-w-0 text-left border px-2 py-1 text-xs font-medium transition-all hover:brightness-95 cursor-pointer mx-0.5',
            seg.continuesBefore ? 'rounded-l-none border-l-0 ml-0' : 'rounded-l-md',
            seg.continuesAfter ? 'rounded-r-none border-r-0 mr-0' : 'rounded-r-md',
            getPriorityColorClasses(seg.task.priority),
            store.selectedTaskId === seg.task.id ? 'ring-2 ring-blue-600' : '',
            seg.task.status === 'done' ? 'opacity-65 line-through' : ''
          ]"
          :title="seg.task.title"
        >
          <span
            @click.stop="store.toggleTaskStatus(seg.task.id)"
            :class="[
              'w-3 h-3 rounded-full border-[1.5px] flex items-center justify-center flex-shrink-0 transition-colors',
              seg.task.status === 'done' ? 'bg-slate-800 border-slate-800 text-white' : 'border-slate-600 hover:border-slate-900'
            ]"
          >
            <Check v-if="seg.task.status === 'done'" class="w-2 h-2 stroke-[3]" />
          </span>
          <span class="truncate flex-1">{{ seg.task.title }}</span>
          <Repeat v-if="seg.task.recurrence_rule" class="w-3 h-3 flex-shrink-0 opacity-75" />
        </button>
      </div>
    </div>

    <!-- Scrollable Time Grid Container -->
    <div class="flex-1 overflow-y-auto relative flex">
      <!-- Left Hour Axis -->
      <div class="w-16 flex-shrink-0 border-r border-slate-200/80 dark:border-slate-800 bg-slate-50/30 dark:bg-slate-900/40 text-[11px] text-slate-400 select-none">
        <!-- Collapsible Night Hours Bar (00:00 - 07:00) -->
        <div
          v-if="store.isNightCollapsed"
          @click="store.toggleNightCollapsed"
          class="h-8 flex items-center justify-between px-2 text-[10px] bg-slate-100 dark:bg-slate-800/60 border-b border-slate-200/80 dark:border-slate-700/80 text-slate-500 hover:bg-slate-200/60 cursor-pointer transition-colors"
          title="Bấm để mở rộng khung giờ đêm (00:00 - 07:00)"
        >
          <span>🌙 00-07h</span>
          <ChevronDown class="w-3 h-3 text-slate-400" />
        </div>

        <div
          v-else
          @click="store.toggleNightCollapsed"
          class="h-7 flex items-center justify-between px-2 text-[10px] bg-slate-100 dark:bg-slate-800/60 border-b border-slate-200/80 dark:border-slate-700/80 text-slate-500 hover:bg-slate-200/60 cursor-pointer transition-colors"
          title="Bấm để thu gọn khung giờ đêm"
        >
          <span>Thu gọn đêm</span>
          <ChevronUp class="w-3 h-3 text-slate-400" />
        </div>

        <!-- Hours List on Left Axis -->
        <div
          v-for="h in hours"
          :key="h"
          class="relative border-b border-slate-200/40 dark:border-slate-800/60 pr-2 text-right"
          :style="{ height: `${HOUR_ROW_HEIGHT}px` }"
        >
          <span class="relative -top-2.5 font-medium">
            <template v-if="h === 12">
              <span class="text-blue-500 font-semibold">Noon</span>
            </template>
            <template v-else>
              {{ String(h).padStart(2, '0') }}:00
            </template>
          </span>
        </div>
      </div>

      <!-- Center Day Columns Grid -->
      <div
        class="flex-1 grid relative"
        :style="{ gridTemplateColumns: `repeat(${days.length}, minmax(0, 1fr))` }"
      >
        <!-- Night Row filler when collapsed -->
        <div
          v-if="store.isNightCollapsed"
          class="col-span-full h-8 bg-slate-100/50 dark:bg-slate-800/30 border-b border-slate-200/80 dark:border-slate-800 flex items-center justify-center text-[11px] text-slate-400 italic cursor-pointer hover:bg-slate-100 dark:hover:bg-slate-800/50 transition-colors"
          @click="store.toggleNightCollapsed"
        >
          Khung giờ đêm (00:00 - 07:00) đã thu gọn. Bấm để hiển thị đầy đủ 24 giờ.
        </div>

        <!-- Per-Day Column -->
        <div
          v-for="d in days"
          :key="d.dateStr"
          class="relative border-r last:border-r-0 border-slate-200/70 dark:border-slate-800"
        >
          <!-- Current Time Indicator Line (for Today's Column) -->
          <div
            v-if="d.isToday && currentTimeIndicator"
            class="absolute left-0 right-0 z-20 pointer-events-none flex items-center"
            :style="{ top: `${currentTimeIndicator.top + (store.isNightCollapsed ? 32 : 0)}px` }"
          >
            <!-- Red Indicator Dot -->
            <div class="w-2.5 h-2.5 rounded-full bg-rose-500 -ml-1.5 shadow-sm ring-2 ring-white dark:ring-slate-900"></div>
            <!-- Red Line -->
            <div class="flex-1 h-0.5 bg-rose-500 shadow-sm"></div>
          </div>

          <!-- Hourly Drop Cells -->
          <div
            v-for="h in hours"
            :key="h"
            @dragover.prevent
            @dragenter.prevent
            @drop="handleCellDrop($event, d, h)"
            @click="handleCellClick(d, h)"
            class="group/cell border-b border-slate-100 dark:border-slate-800/50 hover:bg-blue-50/20 dark:hover:bg-blue-950/10 cursor-pointer transition-colors relative"
            :style="{ height: `${HOUR_ROW_HEIGHT}px` }"
          >
            <!-- Hover + Icon to add task -->
            <button
              type="button"
              class="opacity-0 group-hover/cell:opacity-100 absolute top-1 right-1 p-0.5 rounded text-blue-500 hover:bg-blue-100 dark:hover:bg-blue-900/40 transition-opacity"
              title="Thêm task vào khung giờ này"
            >
              <Plus class="w-3.5 h-3.5" />
            </button>
          </div>

          <!-- Positioned Time-block Cards for this Day -->
          <div
            v-for="task in getTasksForDay(d.date)"
            :key="task.id"
            draggable="true"
            @dragstart="handleCardDragStart($event, task)"
            @click.stop="store.selectTask(task.id)"
            :style="{
              ...getTaskStyle(task, d.date),
              top: `${parseFloat(getTaskStyle(task, d.date).top as string) + (store.isNightCollapsed ? 32 : 0)}px`
            }"
            :class="[
              'absolute left-1 right-1 rounded-[6px] border p-1.5 flex flex-col justify-between overflow-hidden cursor-pointer select-none transition-all group z-10 hover:shadow-md hover:ring-1 hover:ring-slate-400',
              getPriorityColorClasses(task.priority),
              store.selectedTaskId === task.id ? 'ring-2 ring-blue-600 shadow-md' : '',
              task.status === 'done' ? 'opacity-65 line-through' : ''
            ]"
          >
            <!-- Header: Checkbox + Title -->
            <div class="flex items-start gap-1.5 min-w-0">
              <button
                type="button"
                @click.stop="store.toggleTaskStatus(task.id)"
                :class="[
                  'w-3.5 h-3.5 rounded border flex items-center justify-center flex-shrink-0 mt-0.5 transition-colors',
                  task.status === 'done'
                    ? 'bg-slate-800 border-slate-800 text-white'
                    : 'border-slate-600 hover:border-slate-900 text-transparent'
                ]"
              >
                <Check class="w-2.5 h-2.5 stroke-[3]" />
              </button>

              <span class="text-xs font-semibold leading-tight line-clamp-2 select-none">
                {{ task.title }}
              </span>
            </div>

            <!-- Time Range text -->
            <div class="flex items-center gap-1 text-[10px] font-medium text-slate-700/90 mt-0.5 select-none">
              <Clock class="w-2.5 h-2.5 flex-shrink-0" />
              <span class="truncate">{{ formatTaskTime(task) }}</span>
              <Repeat v-if="task.recurrence_rule" class="w-2.5 h-2.5 flex-shrink-0 opacity-75 ml-auto" />
            </div>

            <!-- Bottom Resize Handle -->
            <div
              @mousedown="startResizing($event, task)"
              class="absolute bottom-0 left-0 right-0 h-1.5 cursor-ns-resize opacity-0 group-hover:opacity-100 bg-slate-400/40 hover:bg-blue-500 transition-opacity"
              title="Kéo để thay đổi thời lượng task"
            ></div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

