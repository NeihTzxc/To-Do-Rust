<script setup lang="ts">
import { computed } from 'vue';
import { useTaskStore } from '@/stores/taskStore';
import type { Task, Priority } from '@/types/task';
import { taskDateSpan, localDateKey } from '@/utils/calendarTask';
import { Check, Plus, Repeat } from 'lucide-vue-next';

const emit = defineEmits<{
  (e: 'createAt', payload: { startTime: number; endTime: number; dateStr: string }): void;
}>();

const store = useTaskStore();

const todayStr = localDateKey(new Date());

interface MonthDayCell {
  date: Date;
  dateStr: string;
  dayNum: number;
  isCurrentMonth: boolean;
  isToday: boolean;
}

const monthCells = computed(() => {
  const current = new Date(store.calendarCurrentDate);
  const year = current.getFullYear();
  const month = current.getMonth();

  const firstDay = new Date(year, month, 1);
  const lastDay = new Date(year, month + 1, 0);

  // Day of week: 0 is Sun, 1 is Mon... in Vietnam/TickTick week starts on Monday
  const firstDayOfWeek = firstDay.getDay(); // 0 is Sun, 1 is Mon...
  const prevMonthPadding = firstDayOfWeek === 0 ? 6 : firstDayOfWeek - 1;

  const cells: MonthDayCell[] = [];

  // Padding days from previous month
  const prevMonthLastDay = new Date(year, month, 0).getDate();
  for (let i = prevMonthPadding - 1; i >= 0; i--) {
    const d = new Date(year, month - 1, prevMonthLastDay - i);
    const dateStr = localDateKey(d);
    cells.push({
      date: d,
      dateStr,
      dayNum: d.getDate(),
      isCurrentMonth: false,
      isToday: dateStr === todayStr,
    });
  }

  // Days of current month
  for (let d = 1; d <= lastDay.getDate(); d++) {
    const date = new Date(year, month, d);
    const dateStr = localDateKey(date);
    cells.push({
      date,
      dateStr,
      dayNum: d,
      isCurrentMonth: true,
      isToday: dateStr === todayStr,
    });
  }

  // Padding days to fill 35 or 42 cells (complete 7-day grid)
  const remaining = (7 - (cells.length % 7)) % 7;
  for (let i = 1; i <= remaining; i++) {
    const date = new Date(year, month + 1, i);
    const dateStr = localDateKey(date);
    cells.push({
      date,
      dateStr,
      dayNum: i,
      isCurrentMonth: false,
      isToday: dateStr === todayStr,
    });
  }

  return cells;
});

interface MonthTaskChip {
  task: Task;
  isMultiDay: boolean;
  isStart: boolean;
  isEnd: boolean;
}

function getTasksForCell(date: Date): MonthTaskChip[] {
  const day = localDateKey(date);
  const chips: MonthTaskChip[] = [];

  for (const task of store.tasks) {
    const span = taskDateSpan(task);
    if (!span) continue;
    if (span.startKey <= day && day <= span.endKey) {
      chips.push({
        task,
        isMultiDay: span.isMultiDay,
        isStart: span.startKey === day,
        isEnd: span.endKey === day,
      });
    }
  }

  chips.sort((a, b) => {
    if (a.isMultiDay !== b.isMultiDay) return a.isMultiDay ? -1 : 1;
    return a.task.created_at - b.task.created_at;
  });

  return chips;
}

function getPriorityColorClasses(p: Priority): string {
  switch (p) {
    case 3:
      return 'bg-[#FADBD8] border-[#F1948A] text-slate-900';
    case 2:
      return 'bg-[#85A5FF] border-[#597EF7] text-slate-900';
    case 1:
      return 'bg-[#A8DCE7] border-[#72C8D8] text-slate-900';
    default:
      return 'bg-[#E9EBB2] border-[#D4D785] text-slate-900';
  }
}

function handleDragStart(e: DragEvent, task: Task) {
  if (e.dataTransfer) {
    e.dataTransfer.setData('application/json', JSON.stringify({ taskId: task.id, type: 'month-card' }));
    e.dataTransfer.setData('text/plain', task.id);
    e.dataTransfer.effectAllowed = 'move';
  }
}

async function handleCellDrop(e: DragEvent, cell: MonthDayCell) {
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

  const targetDate = new Date(cell.date);
  targetDate.setHours(9, 0, 0, 0); // default to 09:00 on that date
  const startTime = targetDate.getTime();
  const endTime = startTime + 60 * 60 * 1000;

  await store.updateTaskSchedule(taskId, startTime, endTime);
}

function handleCellClick(cell: MonthDayCell) {
  const targetDate = new Date(cell.date);
  targetDate.setHours(9, 0, 0, 0);
  const startTime = targetDate.getTime();
  const endTime = startTime + 60 * 60 * 1000;

  emit('createAt', {
    startTime,
    endTime,
    dateStr: cell.dateStr,
  });
}
</script>

<template>
  <div class="flex-1 flex flex-col h-full min-h-0 bg-white dark:bg-slate-900 select-none overflow-hidden">
    <!-- Month Day Names Header -->
    <div class="grid grid-cols-7 border-b border-slate-200/80 dark:border-slate-800 bg-slate-50/70 dark:bg-slate-900/90 py-2 text-center text-xs font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wider flex-shrink-0">
      <div>Thứ 2</div>
      <div>Thứ 3</div>
      <div>Thứ 4</div>
      <div>Thứ 5</div>
      <div>Thứ 6</div>
      <div>Thứ 7</div>
      <div>CN</div>
    </div>

    <!-- 7-column Month Grid -->
    <div class="flex-1 grid grid-cols-7 grid-rows-5 md:grid-rows-6 border-b border-slate-200/80 dark:border-slate-800 overflow-y-auto">
      <div
        v-for="cell in monthCells"
        :key="cell.dateStr"
        @dragover.prevent
        @dragenter.prevent
        @drop="handleCellDrop($event, cell)"
        @click="handleCellClick(cell)"
        :class="[
          'border-r border-b border-slate-200/60 dark:border-slate-800/80 p-1.5 flex flex-col min-h-[90px] transition-colors relative group hover:bg-slate-50/60 dark:hover:bg-slate-800/20 cursor-pointer',
          !cell.isCurrentMonth ? 'opacity-40 bg-slate-50/40 dark:bg-slate-950/20' : ''
        ]"
      >
        <!-- Cell Header: Day Number + Add Button -->
        <div class="flex items-center justify-between mb-1">
          <span
            :class="[
              'w-6 h-6 rounded-full flex items-center justify-center text-xs font-semibold',
              cell.isToday
                ? 'bg-[#3B82F6] text-white shadow-sm'
                : 'text-slate-700 dark:text-slate-200'
            ]"
          >
            {{ cell.dayNum }}
          </span>

          <button
            type="button"
            class="opacity-0 group-hover:opacity-100 p-0.5 rounded text-blue-500 hover:bg-blue-50 dark:hover:bg-blue-900/30 transition-opacity"
            title="Thêm task vào ngày này"
          >
            <Plus class="w-3.5 h-3.5" />
          </button>
        </div>

        <!-- Task Chips in this Day -->
        <div class="flex-1 space-y-1 overflow-y-auto max-h-24">
          <div
            v-for="chip in getTasksForCell(cell.date)"
            :key="chip.task.id"
            draggable="true"
            @dragstart="handleDragStart($event, chip.task)"
            @click.stop="store.selectTask(chip.task.id)"
            :class="[
              'py-0.5 border text-[11px] font-medium flex items-center gap-1 truncate shadow-xs cursor-grab active:cursor-grabbing hover:opacity-90',
              !chip.isMultiDay
                ? 'px-1.5 rounded'
                : chip.isStart && chip.isEnd
                  ? 'px-1.5 rounded'
                  : chip.isStart
                    ? 'pl-1.5 pr-1 -mr-1.5 rounded-l border-r-0'
                    : chip.isEnd
                      ? 'pl-1 pr-1.5 -ml-1.5 rounded-r border-l-0'
                      : 'px-1 -mx-1.5 rounded-none border-x-0',
              getPriorityColorClasses(chip.task.priority),
              chip.task.status === 'done' ? 'line-through opacity-60' : '',
              store.selectedTaskId === chip.task.id ? 'ring-2 ring-blue-600' : ''
            ]"
          >
            <button
              type="button"
              @click.stop="store.toggleTaskStatus(chip.task.id)"
              :class="[
                'w-2.5 h-2.5 rounded-sm border flex items-center justify-center flex-shrink-0 transition-colors',
                chip.task.status === 'done'
                  ? 'bg-slate-800 border-slate-800 text-white'
                  : 'border-slate-700 text-transparent'
              ]"
            >
              <Check class="w-2 h-2 stroke-[3]" />
            </button>
            <span class="truncate flex-1">{{ chip.task.title }}</span>
            <Repeat v-if="chip.task.recurrence_rule" class="w-2.5 h-2.5 flex-shrink-0 opacity-75" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

