<script setup lang="ts">
import { computed } from 'vue';
import { WEEKDAY_OPTIONS } from '@/utils/calendarTask';
import { Repeat } from 'lucide-vue-next';

const props = defineProps<{
  modelValue: string | null;
  compact?: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string | null): void;
}>();

const mode = computed<'none' | 'daily' | 'weekly' | 'monthly' | 'custom'>(() => {
  const r = props.modelValue?.trim().toLowerCase() || '';
  if (!r) return 'none';
  if (r === 'daily') return 'daily';
  if (r === 'weekly') return 'weekly';
  if (r === 'monthly') return 'monthly';
  if (r.startsWith('weekly:')) return 'custom';
  return 'none';
});

const selectedWeekdays = computed<number[]>(() => {
  const r = props.modelValue?.trim().toLowerCase() || '';
  if (!r.startsWith('weekly:')) return [];
  return r
    .slice(7)
    .split(',')
    .map((s) => Number(s.trim()))
    .filter((n) => n >= 1 && n <= 7);
});

function onModeChange(e: Event) {
  const val = (e.target as HTMLSelectElement).value;
  if (val === 'none') {
    emit('update:modelValue', null);
  } else if (val === 'daily' || val === 'weekly' || val === 'monthly') {
    emit('update:modelValue', val);
  } else if (val === 'custom') {
    const todayWd = new Date().getDay() === 0 ? 7 : new Date().getDay();
    emit('update:modelValue', `weekly:${todayWd}`);
  }
}

function toggleWeekday(day: number) {
  const current = new Set(selectedWeekdays.value);
  if (current.has(day)) {
    if (current.size > 1) current.delete(day);
  } else {
    current.add(day);
  }
  const sorted = Array.from(current).sort((a, b) => a - b);
  emit('update:modelValue', `weekly:${sorted.join(',')}`);
}
</script>

<template>
  <div class="space-y-2">
    <div class="flex items-center justify-between gap-2">
      <div v-if="!compact" class="flex items-center gap-2 text-slate-500 dark:text-slate-400 text-sm">
        <Repeat class="w-4 h-4" />
        <span>Lặp lại</span>
      </div>
      <select
        :value="mode"
        @change="onModeChange"
        :class="[
          'bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-md text-slate-700 dark:text-slate-200 outline-none focus:border-blue-500',
          compact ? 'w-full px-2 py-1 text-xs' : 'px-2.5 py-1.5 text-sm'
        ]"
      >
        <option value="none">Không lặp lại</option>
        <option value="daily">Hàng ngày</option>
        <option value="weekly">Hàng tuần</option>
        <option value="monthly">Hàng tháng</option>
        <option value="custom">Tùy chỉnh thứ trong tuần</option>
      </select>
    </div>

    <!-- Custom Weekday Toggle Pills -->
    <div v-if="mode === 'custom'" class="flex items-center gap-1 flex-wrap justify-end">
      <button
        v-for="w in WEEKDAY_OPTIONS"
        :key="w.value"
        type="button"
        @click="toggleWeekday(w.value)"
        :class="[
          'w-7 h-7 rounded-full text-[11px] font-semibold transition-colors cursor-pointer',
          selectedWeekdays.includes(w.value)
            ? 'bg-blue-600 text-white shadow-xs'
            : 'bg-slate-100 dark:bg-slate-800 text-slate-500 dark:text-slate-400 hover:bg-slate-200 dark:hover:bg-slate-700'
        ]"
      >
        {{ w.label }}
      </button>
    </div>
  </div>
</template>
