import type { Task } from '../types/task';

export function localDateKey(date: Date): string {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
}

// The due date is the calendar day. A saved time block supplies its time of day.
export function calendarInterval(task: Task): { start: number; end: number } | null {
  if (task.start_time == null || task.end_time == null || task.end_time <= task.start_time) return null;
  if (task.due_date == null) return { start: task.start_time, end: task.end_time };

  const due = new Date(task.due_date);
  const start = new Date(task.start_time);
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
