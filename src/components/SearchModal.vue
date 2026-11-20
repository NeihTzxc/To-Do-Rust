<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue';
import { useTaskStore } from '@/stores/taskStore';
import type { Task, Priority } from '@/types/task';
import {
  Search,
  X,
  Check,
  Calendar,
  Clock,
  Flag,
  ArrowRight,
  Sparkles,
} from 'lucide-vue-next';

const store = useTaskStore();

const searchInput = ref<HTMLInputElement | null>(null);
const query = ref('');
const results = ref<Task[]>([]);
const isLoading = ref(false);
const selectedIndex = ref(0);
const activeFilter = ref<'all' | 'active' | 'done' | 'scheduled'>('all');

let debounceTimeout: any = null;

// Watch for modal open state to auto-focus and load recent tasks
watch(
  () => store.isSearchModalOpen,
  async (isOpen) => {
    if (isOpen) {
      query.value = '';
      activeFilter.value = 'all';
      selectedIndex.value = 0;
      await fetchResults('');
      nextTick(() => {
        searchInput.value?.focus();
      });
    }
  }
);

async function fetchResults(q: string) {
  isLoading.value = true;
  try {
    const list = await store.searchTasksRemote(q);
    results.value = list;
    selectedIndex.value = 0;
  } finally {
    isLoading.value = false;
  }
}

function onInput() {
  if (debounceTimeout) clearTimeout(debounceTimeout);
  debounceTimeout = setTimeout(() => {
    fetchResults(query.value);
  }, 150);
}

// Filtered results by active quick filter chip
const filteredResults = computed(() => {
  let list = results.value;
  switch (activeFilter.value) {
    case 'active':
      list = list.filter((t) => t.status !== 'done');
      break;
    case 'done':
      list = list.filter((t) => t.status === 'done');
      break;
    case 'scheduled':
      list = list.filter((t) => !!(t.start_time && t.end_time));
      break;
    case 'all':
    default:
      break;
  }
  return list;
});

// Keyboard Navigation
function handleKeydown(e: KeyboardEvent) {
  if (!store.isSearchModalOpen) return;

  if (e.key === 'ArrowDown') {
    e.preventDefault();
    if (filteredResults.value.length > 0) {
      selectedIndex.value = (selectedIndex.value + 1) % filteredResults.value.length;
      scrollToActive();
    }
  } else if (e.key === 'ArrowUp') {
    e.preventDefault();
    if (filteredResults.value.length > 0) {
      selectedIndex.value =
        (selectedIndex.value - 1 + filteredResults.value.length) % filteredResults.value.length;
      scrollToActive();
    }
  } else if (e.key === 'Enter') {
    e.preventDefault();
    if (filteredResults.value[selectedIndex.value]) {
      selectAndNavigate(filteredResults.value[selectedIndex.value]);
    }
  } else if (e.key === 'Escape') {
    store.closeSearchModal();
  }
}

function scrollToActive() {
  nextTick(() => {
    const activeEl = document.querySelector('.search-item-active');
    if (activeEl) {
      activeEl.scrollIntoView({ block: 'nearest' });
    }
  });
}

function selectAndNavigate(task: Task) {
  store.closeSearchModal();

  // If task has calendar schedule, navigate to Calendar view and focus that date
  if (task.start_time && task.end_time) {
    store.setNavTab('calendar');
    store.setCalendarDate(task.start_time);
  } else {
    store.setNavTab('tasks');
  }

  // Open Task Detail Drawer
  store.selectTask(task.id);
}

// Highlight matched query substring in text
function highlightMatch(text: string, q: string): string {
  if (!q.trim()) return escapeHtml(text);

  const cleanQ = q.trim();
  // Regex case-insensitive match
  const escapedQ = cleanQ.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const regex = new RegExp(`(${escapedQ})`, 'gi');

  const escapedText = escapeHtml(text);
  return escapedText.replace(
    regex,
    '<mark class="bg-blue-100 dark:bg-blue-900/60 text-blue-700 dark:text-blue-300 font-bold px-0.5 rounded">$1</mark>'
  );
}

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

function getPriorityBorder(p: Priority): string {
  switch (p) {
    case 3:
      return 'border-l-rose-500 bg-rose-50/20';
    case 2:
      return 'border-l-blue-500 bg-blue-50/20';
    case 1:
      return 'border-l-teal-500 bg-teal-50/20';
    default:
      return 'border-l-slate-300 bg-transparent';
  }
}

function formatTaskTime(task: Task): string | null {
  if (task.start_time && task.end_time) {
    const s = new Date(task.start_time);
    const e = new Date(task.end_time);
    const sStr = `${String(s.getHours()).padStart(2, '0')}:${String(s.getMinutes()).padStart(2, '0')}`;
    const eStr = `${String(e.getHours()).padStart(2, '0')}:${String(e.getMinutes()).padStart(2, '0')}`;
    const dateStr = s.toLocaleDateString('vi-VN', { day: '2-digit', month: '2-digit' });
    return `${dateStr} ${sStr}-${eStr}`;
  }
  if (task.due_date) {
    return new Date(task.due_date).toLocaleDateString('vi-VN', { day: '2-digit', month: '2-digit' });
  }
  return null;
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
});
</script>

<template>
  <div
    v-if="store.isSearchModalOpen"
    class="fixed inset-0 z-50 flex items-start justify-center pt-20 px-4 bg-black/50 backdrop-blur-xs select-none transition-all"
    @click.self="store.closeSearchModal"
  >
    <!-- Spotlight Command Palette Dialog Container -->
    <div
      class="w-full max-w-2xl bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-2xl shadow-2xl flex flex-col overflow-hidden max-h-[75vh] animate-in fade-in zoom-in-95 duration-150"
    >
      <!-- Top Search Input Bar -->
      <div class="flex items-center px-4 py-3.5 border-b border-slate-200/80 dark:border-slate-800 gap-3">
        <Search class="w-5 h-5 text-slate-400 flex-shrink-0" />
        <input
          ref="searchInput"
          v-model="query"
          type="text"
          placeholder="Tìm công việc, ghi chú, lịch trình... (Ctrl + K)"
          @input="onInput"
          class="flex-1 bg-transparent text-sm md:text-base text-slate-800 dark:text-slate-100 placeholder-slate-400 outline-none"
        />
        <button
          v-if="query"
          type="button"
          @click="query = ''; fetchResults('')"
          class="p-1 rounded-md text-slate-400 hover:text-slate-600 dark:hover:text-slate-200"
        >
          <X class="w-4 h-4" />
        </button>
        <span
          class="hidden sm:inline px-2 py-0.5 text-[10px] font-semibold tracking-wider text-slate-400 bg-slate-100 dark:bg-slate-800 rounded border border-slate-200 dark:border-slate-700"
        >
          ESC
        </span>
      </div>

      <!-- Quick Filter Chips -->
      <div class="flex items-center gap-1.5 px-4 py-2 border-b border-slate-100 dark:border-slate-800/80 bg-slate-50/60 dark:bg-slate-950/40 overflow-x-auto text-xs">
        <button
          v-for="chip in [
            { id: 'all', label: 'Tất cả' },
            { id: 'active', label: 'Chưa xong' },
            { id: 'done', label: 'Đã xong' },
            { id: 'scheduled', label: 'Có lịch hẹn' },
          ]"
          :key="chip.id"
          type="button"
          @click="activeFilter = chip.id as any; selectedIndex = 0"
          :class="[
            'px-2.5 py-1 rounded-lg font-medium transition-all text-xs flex-shrink-0',
            activeFilter === chip.id
              ? 'bg-blue-600 text-white shadow-xs font-semibold'
              : 'text-slate-600 dark:text-slate-400 hover:bg-slate-200/60 dark:hover:bg-slate-800 hover:text-slate-900 dark:hover:text-slate-200'
          ]"
        >
          {{ chip.label }}
        </button>

        <span class="ml-auto text-[11px] text-slate-400 font-medium">
          {{ filteredResults.length }} kết quả
        </span>
      </div>

      <!-- Results Body -->
      <div class="flex-1 overflow-y-auto p-2 space-y-1 divide-y divide-slate-100 dark:divide-slate-800/40">
        <!-- Section Header (when empty query) -->
        <div
          v-if="!query.trim()"
          class="px-3 py-1.5 text-[11px] font-bold text-slate-400 uppercase tracking-wider flex items-center gap-1.5"
        >
          <Sparkles class="w-3 h-3 text-amber-500" />
          <span>Tìm kiếm gần đây / Công việc mới cập nhật</span>
        </div>

        <!-- Empty State -->
        <div
          v-if="filteredResults.length === 0 && !isLoading"
          class="py-12 text-center text-slate-400 space-y-2"
        >
          <Search class="w-8 h-8 mx-auto text-slate-300 dark:text-slate-600 stroke-[1.5]" />
          <p class="text-sm font-medium">Không tìm thấy công việc nào khớp với "{{ query }}"</p>
          <p class="text-xs text-slate-400">Thử tìm bằng từ khóa khác hoặc gõ tiếng Việt không dấu</p>
        </div>

        <!-- Result Item Rows -->
        <div
          v-for="(task, idx) in filteredResults"
          :key="task.id"
          @mouseenter="selectedIndex = idx"
          @click="selectAndNavigate(task)"
          :class="[
            'group flex items-center gap-3 p-2.5 rounded-xl border-l-4 cursor-pointer transition-all',
            getPriorityBorder(task.priority),
            selectedIndex === idx
              ? 'search-item-active bg-blue-50/80 dark:bg-blue-950/50 shadow-xs ring-1 ring-blue-500/30'
              : 'hover:bg-slate-50 dark:hover:bg-slate-800/50'
          ]"
        >
          <!-- Status Checkbox -->
          <button
            type="button"
            @click.stop="store.toggleTaskStatus(task.id)"
            :class="[
              'w-4 h-4 rounded border flex items-center justify-center flex-shrink-0 transition-colors',
              task.status === 'done'
                ? 'bg-emerald-500 border-emerald-500 text-white'
                : 'border-slate-300 dark:border-slate-600 hover:border-blue-500 text-transparent'
            ]"
          >
            <Check class="w-3 h-3 stroke-[2.5]" />
          </button>

          <!-- Task Details -->
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <span
                class="text-sm font-medium text-slate-800 dark:text-slate-100 truncate"
                :class="task.status === 'done' ? 'line-through opacity-60' : ''"
                v-html="highlightMatch(task.title, query)"
              />

              <!-- Priority Flag Badge -->
              <span
                v-if="task.priority === 3"
                class="px-1.5 py-0.2 rounded text-[10px] font-semibold bg-rose-100 dark:bg-rose-950/60 text-rose-600 dark:text-rose-400 flex items-center gap-0.5"
              >
                <Flag class="w-2.5 h-2.5 fill-current" /> Cao
              </span>
              <span
                v-else-if="task.priority === 2"
                class="px-1.5 py-0.2 rounded text-[10px] font-semibold bg-blue-100 dark:bg-blue-950/60 text-blue-600 dark:text-blue-400"
              >
                Vừa
              </span>
            </div>

            <!-- Description match snippet if available -->
            <p
              v-if="task.description"
              class="text-xs text-slate-500 dark:text-slate-400 truncate mt-0.5"
              v-html="highlightMatch(task.description, query)"
            />
          </div>

          <!-- Time / Schedule Badge -->
          <div class="flex items-center gap-2 text-xs flex-shrink-0">
            <span
              v-if="formatTaskTime(task)"
              class="flex items-center gap-1 text-[11px] text-slate-500 dark:text-slate-400 bg-slate-100 dark:bg-slate-800 px-2 py-0.5 rounded-md"
            >
              <Clock v-if="task.start_time" class="w-3 h-3 text-blue-500" />
              <Calendar v-else class="w-3 h-3 text-amber-500" />
              <span>{{ formatTaskTime(task) }}</span>
            </span>

            <!-- Status Label -->
            <span
              class="text-[10px] font-semibold px-2 py-0.5 rounded-full"
              :class="[
                task.status === 'done'
                  ? 'bg-emerald-100 dark:bg-emerald-950/50 text-emerald-700 dark:text-emerald-300'
                  : task.status === 'in_progress'
                  ? 'bg-amber-100 dark:bg-amber-950/50 text-amber-700 dark:text-amber-300'
                  : 'bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-400'
              ]"
            >
              {{ task.status === 'done' ? 'Đã xong' : task.status === 'in_progress' ? 'Đang làm' : 'Cần làm' }}
            </span>

            <ArrowRight
              class="w-4 h-4 text-slate-400 group-hover:text-blue-500 transition-colors opacity-0 group-hover:opacity-100"
            />
          </div>
        </div>
      </div>

      <!-- Footer: Keyboard Shortcuts Hint -->
      <div class="px-4 py-2.5 bg-slate-50 dark:bg-slate-950/60 border-t border-slate-200/60 dark:border-slate-800 text-[11px] text-slate-400 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <span class="flex items-center gap-1">
            <kbd class="px-1.5 py-0.5 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-[10px] font-mono">↑</kbd>
            <kbd class="px-1.5 py-0.5 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-[10px] font-mono">↓</kbd>
            di chuyển
          </span>
          <span class="flex items-center gap-1">
            <kbd class="px-1.5 py-0.5 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-[10px] font-mono">↵</kbd>
            chọn
          </span>
          <span class="flex items-center gap-1">
            <kbd class="px-1.5 py-0.5 bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded text-[10px] font-mono">ESC</kbd>
            đóng
          </span>
        </div>

        <span class="hidden sm:inline text-slate-400">
          Tìm kiếm tiếng Việt không dấu
        </span>
      </div>
    </div>
  </div>
</template>
