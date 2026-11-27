import type { Task } from '../types/task';

export const WEEKDAY_OPTIONS: { value: number; label: string }[] = [
  { value: 1, label: 'T2' },
  { value: 2, label: 'T3' },
  { value: 3, label: 'T4' },
  { value: 4, label: 'T5' },
  { value: 5, label: 'T6' },
  { value: 6, label: 'T7' },
  { value: 7, label: 'CN' },
];

export function localDateKey(date: Date): string {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
}

export function parseDateKey(key: string): Date {
  const [y, m, d] = key.split('-').map(Number);
  return new Date(y, m - 1, d, 0, 0, 0, 0);
}

export function startOfDayMs(date: Date): number {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate(), 0, 0, 0, 0).getTime();
}

export function endOfDayMs(date: Date): number {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate(), 23, 59, 59, 999).getTime();
}

export function isMultiDayTask(task: Task): boolean {
  if (task.start_time == null || task.end_time == null) return false;
  return localDateKey(new Date(task.start_time)) !== localDateKey(new Date(task.end_time));
}

export function isAllDayOrMultiDay(task: Task): boolean {
  if (task.is_all_day) return true;
  if (isMultiDayTask(task)) return true;
  // Task có due_date nhưng không có khung giờ cụ thể
  if (task.due_date != null && (task.start_time == null || task.end_time == null)) return true;
  return false;
}

export function taskDateSpan(task: Task): { startKey: string; endKey: string; isMultiDay: boolean } | null {
  if (task.start_time != null && task.end_time != null && task.end_time >= task.start_time) {
    const startKey = localDateKey(new Date(task.start_time));
    const endKey = localDateKey(new Date(task.end_time));
    return { startKey, endKey, isMultiDay: startKey !== endKey };
  }
  if (task.start_time != null) {
    const k = localDateKey(new Date(task.start_time));
    return { startKey: k, endKey: k, isMultiDay: false };
  }
  if (task.due_date != null) {
    const k = localDateKey(new Date(task.due_date));
    return { startKey: k, endKey: k, isMultiDay: false };
  }
  return null;
}

// Chỉ trả về khoảng giờ trong ngày cho task Time-blocking (không phải All-day / Multi-day)
export function calendarInterval(task: Task): { start: number; end: number } | null {
  if (task.is_all_day) return null;
  if (task.start_time == null || task.end_time == null || task.end_time <= task.start_time) return null;

  const start = new Date(task.start_time);
  const end = new Date(task.end_time);
  if (localDateKey(start) !== localDateKey(end)) {
    // Multi-day task hiển thị ở thanh ngang phía trên thay vì chiếm cột giờ
    return null;
  }

  if (task.due_date == null) return { start: task.start_time, end: task.end_time };

  const due = new Date(task.due_date);
  if (localDateKey(due) === localDateKey(start)) {
    return { start: task.start_time, end: task.end_time };
  }

  const projectedStart = new Date(
    due.getFullYear(), due.getMonth(), due.getDate(),
    start.getHours(), start.getMinutes(), start.getSeconds(), start.getMilliseconds(),
  ).getTime();
  return { start: projectedStart, end: projectedStart + task.end_time - task.start_time };
}

export function calendarDay(task: Task): string | null {
  if (task.due_date != null) return localDateKey(new Date(task.due_date));
  if (task.start_time != null) return localDateKey(new Date(task.start_time));
  return null;
}

export function formatRecurrenceLabel(rule: string | null | undefined): string | null {
  if (!rule) return null;
  const r = rule.trim().toLowerCase();
  if (!r) return null;
  if (r === 'daily') return 'Hàng ngày';
  if (r === 'weekly') return 'Hàng tuần';
  if (r === 'monthly') return 'Hàng tháng';
  if (r.startsWith('weekly:')) {
    const days = r
      .slice(7)
      .split(',')
      .map((s) => Number(s.trim()))
      .filter((n) => n >= 1 && n <= 7)
      .sort((a, b) => a - b);
    if (days.length === 0) return 'Hàng tuần';
    return days
      .map((d) => WEEKDAY_OPTIONS.find((w) => w.value === d)?.label || '')
      .filter(Boolean)
      .join(', ');
  }
  return rule;
}
