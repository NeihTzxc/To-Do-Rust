import assert from 'node:assert/strict';
import test from 'node:test';
import { calendarDay, calendarInterval, localDateKey } from '../src/utils/calendarTask.ts';

const at = (year, month, day, hour = 0, minute = 0) => new Date(year, month - 1, day, hour, minute).getTime();

test('a task with only a due date belongs to that calendar day', () => {
  const birthday = { due_date: at(2026, 11, 27, 23, 59), start_time: null, end_time: null };
  assert.equal(calendarDay(birthday), '2026-11-27');
  assert.equal(calendarInterval(birthday), null);
});

test('a due date takes precedence over a stale time block from another month', () => {
  const christmas = {
    due_date: at(2026, 12, 25, 23, 59),
    start_time: at(2026, 11, 25, 9),
    end_time: at(2026, 11, 25, 10),
  };
  assert.equal(calendarDay(christmas), '2026-12-25');
  assert.deepEqual(calendarInterval(christmas), {
    start: at(2026, 12, 25, 9),
    end: at(2026, 12, 25, 10),
  });
});

test('a scheduled task without a due date stays on its scheduled day', () => {
  const task = { due_date: null, start_time: at(2026, 11, 23, 8, 30), end_time: at(2026, 11, 23, 18) };
  assert.equal(calendarDay(task), '2026-11-23');
  assert.deepEqual(calendarInterval(task), { start: task.start_time, end: task.end_time });
});

test('local day keys do not roll back at midnight in UTC+7', () => {
  assert.equal(localDateKey(new Date(at(2026, 11, 27))), '2026-11-27');
});
