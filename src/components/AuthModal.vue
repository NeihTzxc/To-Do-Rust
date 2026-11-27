<script setup lang="ts">
import { ref, computed } from 'vue';
import { useTaskStore } from '@/stores/taskStore';
import {
  X,
  User,
  Mail,
  Lock,
  Cloud,
  CheckCircle2,
  CloudAlert,
  CloudOff,
  RefreshCw,
  LogOut,
  AlertCircle,
  Eye,
  EyeOff,
  Minimize2,
  Power,
  Bell,
} from 'lucide-vue-next';

const store = useTaskStore();

const activeTab = ref<'login' | 'register'>('login');
const email = ref('');
const password = ref('');
const showPassword = ref(false);

const userInitial = computed(() => {
  if (store.currentUser?.email) {
    return store.currentUser.email.charAt(0).toUpperCase();
  }
  return 'U';
});

const formattedLastSync = computed(() => {
  if (!store.syncState.last_synced_at) return 'Chưa có';
  const d = new Date(store.syncState.last_synced_at);
  return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' }) + ' ' + d.toLocaleDateString();
});

async function handleSubmit() {
  if (!email.value || !password.value) return;
  if (activeTab.value === 'login') {
    const ok = await store.login(email.value.trim(), password.value);
    if (ok) {
      password.value = '';
    }
  } else {
    const ok = await store.register(email.value.trim(), password.value);
    if (ok) {
      password.value = '';
    }
  }
}

async function handleLogout() {
  await store.logout();
}

function handleClose() {
  store.closeAuthModal();
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="store.isAuthModalOpen"
      class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/45 backdrop-blur-xs animate-in fade-in duration-150"
      @click.self="handleClose"
      @keydown.esc="handleClose"
    >
      <div
        class="w-full max-w-md bg-white dark:bg-slate-900 border border-slate-200/90 dark:border-slate-800 rounded-2xl shadow-2xl overflow-hidden flex flex-col transition-all duration-200"
      >
        <!-- Modal Top Bar -->
        <div class="flex items-center justify-between px-6 pt-5 pb-3 border-b border-slate-100 dark:border-slate-800/80">
          <div v-if="!store.isLoggedIn" class="flex items-center gap-2">
            <button
              type="button"
              @click="activeTab = 'login'"
              :class="[
                'text-sm font-semibold pb-1 border-b-2 transition-colors',
                activeTab === 'login'
                  ? 'border-blue-600 text-blue-600 dark:border-blue-400 dark:text-blue-400'
                  : 'border-transparent text-slate-500 hover:text-slate-800 dark:hover:text-slate-300'
              ]"
            >
              Đăng nhập
            </button>
            <span class="text-slate-300 dark:text-slate-700 px-1">/</span>
            <button
              type="button"
              @click="activeTab = 'register'"
              :class="[
                'text-sm font-semibold pb-1 border-b-2 transition-colors',
                activeTab === 'register'
                  ? 'border-blue-600 text-blue-600 dark:border-blue-400 dark:text-blue-400'
                  : 'border-transparent text-slate-500 hover:text-slate-800 dark:hover:text-slate-300'
              ]"
            >
              Đăng ký
            </button>
          </div>
          <div v-else class="flex items-center gap-2">
            <User class="w-4 h-4 text-blue-600 dark:text-blue-400" />
            <h2 class="text-sm font-semibold text-slate-800 dark:text-slate-100">
              Tài khoản & Đồng bộ Đám mây
            </h2>
          </div>

          <button
            type="button"
            @click="handleClose"
            class="p-1 rounded-lg text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        <!-- Body Content -->
        <div class="p-6">
          <!-- GUEST / NOT LOGGED IN FORM -->
          <div v-if="!store.isLoggedIn" class="space-y-4">
            <!-- Informational Banner -->
            <div class="p-3 bg-blue-50/70 dark:bg-blue-950/40 border border-blue-100 dark:border-blue-900/50 rounded-xl flex items-start gap-3 text-xs text-blue-800 dark:text-blue-300">
              <Cloud class="w-4 h-4 text-blue-600 dark:text-blue-400 flex-shrink-0 mt-0.5" />
              <div>
                <p class="font-medium">Kiến trúc Local-first (SQLite Cục bộ):</p>
                <p class="text-slate-600 dark:text-slate-400 mt-0.5">
                  Ứng dụng hoạt động 0ms ngay trên máy tính của bạn. Khi đăng nhập, công việc cục bộ sẽ tự động kết nối và đồng bộ với Supabase Cloud.
                </p>
              </div>
            </div>

            <!-- Error Banner -->
            <div
              v-if="store.authError"
              class="p-3 bg-rose-50 dark:bg-rose-950/50 border border-rose-200 dark:border-rose-900 rounded-xl flex items-start gap-2.5 text-xs text-rose-700 dark:text-rose-300"
            >
              <AlertCircle class="w-4 h-4 flex-shrink-0 mt-0.5 text-rose-500" />
              <div class="flex-1 break-words">
                {{ store.authError }}
              </div>
            </div>

            <form @submit.prevent="handleSubmit" class="space-y-3.5">
              <div>
                <label class="block text-xs font-medium text-slate-600 dark:text-slate-300 mb-1.5">
                  Địa chỉ Email
                </label>
                <div class="relative">
                  <Mail class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
                  <input
                    v-model="email"
                    type="email"
                    required
                    placeholder="name@example.com"
                    class="w-full pl-9 pr-3 py-2 text-sm bg-slate-50 dark:bg-slate-800/80 border border-slate-200 dark:border-slate-700 rounded-xl text-slate-800 dark:text-slate-100 placeholder-slate-400 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500 transition-all"
                  />
                </div>
              </div>

              <div>
                <label class="block text-xs font-medium text-slate-600 dark:text-slate-300 mb-1.5">
                  Mật khẩu
                </label>
                <div class="relative">
                  <Lock class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
                  <input
                    v-model="password"
                    :type="showPassword ? 'text' : 'password'"
                    required
                    minlength="6"
                    placeholder="Tối thiểu 6 ký tự"
                    class="w-full pl-9 pr-10 py-2 text-sm bg-slate-50 dark:bg-slate-800/80 border border-slate-200 dark:border-slate-700 rounded-xl text-slate-800 dark:text-slate-100 placeholder-slate-400 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500 transition-all"
                  />
                  <button
                    type="button"
                    @click="showPassword = !showPassword"
                    class="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200"
                  >
                    <EyeOff v-if="showPassword" class="w-4 h-4" />
                    <Eye v-else class="w-4 h-4" />
                  </button>
                </div>
              </div>

              <button
                type="submit"
                :disabled="store.authLoading"
                class="w-full mt-2 py-2.5 px-4 bg-blue-600 hover:bg-blue-700 active:bg-blue-800 disabled:opacity-60 text-white font-medium text-sm rounded-xl shadow-sm shadow-blue-500/20 flex items-center justify-center gap-2 transition-all cursor-pointer"
              >
                <RefreshCw v-if="store.authLoading" class="w-4 h-4 animate-spin" />
                <span>
                  {{ store.authLoading ? 'Đang xử lý...' : activeTab === 'login' ? 'Đăng nhập' : 'Tạo tài khoản mới' }}
                </span>
              </button>
            </form>
          </div>

          <!-- LOGGED IN USER PROFILE & SYNC DASHBOARD -->
          <div v-else class="space-y-5">
            <!-- User Info Badge -->
            <div class="flex items-center gap-3.5 p-3.5 bg-slate-50 dark:bg-slate-800/60 rounded-xl border border-slate-200/60 dark:border-slate-700/60">
              <div class="w-11 h-11 rounded-xl bg-gradient-to-tr from-blue-600 to-indigo-500 flex items-center justify-center text-white font-bold text-base shadow-xs flex-shrink-0">
                {{ userInitial }}
              </div>
              <div class="flex-1 min-w-0">
                <div class="text-sm font-semibold text-slate-800 dark:text-slate-100 truncate">
                  {{ store.currentUser?.email }}
                </div>
                <div class="flex items-center gap-1.5 mt-0.5 text-xs text-emerald-600 dark:text-emerald-400 font-medium">
                  <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
                  Đã kết nối Supabase Cloud
                </div>
              </div>
            </div>

            <!-- Sync Status Card (TickTick-like) -->
            <div class="border border-slate-200/80 dark:border-slate-800 rounded-xl p-4 bg-slate-50/50 dark:bg-slate-800/30 space-y-3">
              <div class="flex items-center justify-between">
                <span class="text-xs font-medium text-slate-500 dark:text-slate-400">Trạng thái đồng bộ</span>
                <div class="flex items-center gap-1.5 text-xs font-semibold">
                  <!-- Synced -->
                  <template v-if="store.syncState.state === 'synced' && !store.syncTriggerLoading">
                    <CheckCircle2 class="w-4 h-4 text-emerald-500" />
                    <span class="text-emerald-600 dark:text-emerald-400">Đã đồng bộ</span>
                  </template>
                  <!-- Syncing -->
                  <template v-else-if="store.syncState.state === 'syncing' || store.syncTriggerLoading">
                    <RefreshCw class="w-4 h-4 text-blue-500 animate-spin" />
                    <span class="text-blue-600 dark:text-blue-400">Đang đồng bộ...</span>
                  </template>
                  <!-- Error -->
                  <template v-else-if="store.syncState.state === 'error'">
                    <CloudAlert class="w-4 h-4 text-rose-500" />
                    <span class="text-rose-600 dark:text-rose-400">Lỗi kết nối</span>
                  </template>
                  <!-- Offline -->
                  <template v-else>
                    <CloudOff class="w-4 h-4 text-amber-500" />
                    <span class="text-amber-600 dark:text-amber-400">Cục bộ (Chờ đồng bộ)</span>
                  </template>
                </div>
              </div>

              <div class="grid grid-cols-2 gap-2 pt-2 border-t border-slate-200/60 dark:border-slate-800 text-xs">
                <div>
                  <div class="text-slate-400 dark:text-slate-500">Thay đổi chờ gửi:</div>
                  <div class="font-medium text-slate-700 dark:text-slate-200 mt-0.5">
                    {{ store.syncState.pending_count }} task
                  </div>
                </div>
                <div>
                  <div class="text-slate-400 dark:text-slate-500">Đồng bộ gần nhất:</div>
                  <div class="font-medium text-slate-700 dark:text-slate-200 mt-0.5 truncate" :title="formattedLastSync">
                    {{ formattedLastSync }}
                  </div>
                </div>
              </div>

              <!-- Button Đồng bộ ngay -->
              <button
                type="button"
                @click="store.triggerSyncNow"
                :disabled="store.syncTriggerLoading"
                class="w-full mt-2 py-2 px-3 bg-white dark:bg-slate-800 hover:bg-slate-100 dark:hover:bg-slate-700/80 active:bg-slate-200 border border-slate-200 dark:border-slate-700 rounded-lg text-xs font-semibold text-slate-700 dark:text-slate-200 shadow-xs flex items-center justify-center gap-2 transition-all cursor-pointer disabled:opacity-60"
              >
                <RefreshCw class="w-3.5 h-3.5 text-blue-600 dark:text-blue-400" :class="store.syncTriggerLoading ? 'animate-spin' : ''" />
                <span>{{ store.syncTriggerLoading ? 'Đang gửi & kéo dữ liệu...' : 'Đồng bộ ngay với Cloud' }}</span>
              </button>
            </div>

            <!-- Error Details if any -->
            <div
              v-if="store.syncState.error_message"
              class="p-2.5 bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-900/50 rounded-lg text-xs text-rose-600 dark:text-rose-300"
            >
              {{ store.syncState.error_message }}
            </div>

            <!-- Logout Button -->
            <div class="pt-2 flex flex-col gap-2">
              <button
                type="button"
                @click="handleLogout"
                :disabled="store.authLoading"
                class="w-full py-2 px-3 border border-rose-200 dark:border-rose-900/60 hover:bg-rose-50 dark:hover:bg-rose-950/40 text-rose-600 dark:text-rose-400 text-xs font-semibold rounded-xl flex items-center justify-center gap-2 transition-colors cursor-pointer disabled:opacity-60"
              >
                <LogOut class="w-3.5 h-3.5" />
                <span>Đăng xuất khỏi thiết bị này</span>
              </button>
              <p class="text-[11px] text-center text-slate-400 dark:text-slate-500">
                Lưu ý: Mọi công việc lưu trên SQLite cục bộ vẫn được giữ nguyên an toàn.
              </p>
            </div>
          </div>

          <!-- System Tray & Notification Settings (Dùng chung cho cả Guest & User) -->
          <div class="mt-5 pt-4 border-t border-slate-100 dark:border-slate-800 space-y-3">
            <div class="flex items-center justify-between">
              <div>
                <div class="text-xs font-semibold text-slate-700 dark:text-slate-200 flex items-center gap-1.5">
                  <Bell class="w-3.5 h-3.5 text-amber-500" />
                  <span>Thông báo nhắc việc khi tới giờ</span>
                </div>
                <p class="text-[11px] text-slate-400 dark:text-slate-500 mt-0.5">
                  Gửi thông báo Desktop khi đến giờ bắt đầu của task (khi mở app)
                </p>
              </div>
              <div class="flex items-center gap-2">
                <button
                  type="button"
                  @click="store.sendTestNotification"
                  class="px-2 py-0.5 text-[11px] font-medium rounded border border-slate-200 dark:border-slate-700 text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 cursor-pointer"
                  title="Gửi thử thông báo Desktop"
                >
                  Thử
                </button>
                <label class="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    :checked="store.notificationsEnabled"
                    @change="(e: any) => store.setNotificationsEnabled(e.target.checked)"
                    class="sr-only peer"
                  />
                  <div class="w-9 h-5 bg-slate-300 peer-focus:outline-hidden rounded-full peer dark:bg-slate-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all dark:border-slate-600 peer-checked:bg-blue-600"></div>
                </label>
              </div>
            </div>

            <div class="flex items-center justify-between">
              <div>
                <div class="text-xs font-semibold text-slate-700 dark:text-slate-200 flex items-center gap-1.5">
                  <Minimize2 class="w-3.5 h-3.5 text-blue-500" />
                  <span>Thu nhỏ xuống khay hệ thống (Tray)</span>
                </div>
                <p class="text-[11px] text-slate-400 dark:text-slate-500 mt-0.5">
                  Khi bấm 'X', ứng dụng ẩn xuống khay để tiếp tục đồng bộ ngầm
                </p>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  :checked="store.closeToTray"
                  @change="(e: any) => store.setCloseToTray(e.target.checked)"
                  class="sr-only peer"
                />
                <div class="w-9 h-5 bg-slate-300 peer-focus:outline-hidden rounded-full peer dark:bg-slate-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all dark:border-slate-600 peer-checked:bg-blue-600"></div>
              </label>
            </div>

            <!-- Nút Thoát hẳn ứng dụng -->
            <button
              type="button"
              @click="store.exitApp"
              class="w-full py-2 px-3 bg-slate-50 hover:bg-rose-50 dark:bg-slate-800/60 dark:hover:bg-rose-950/40 border border-slate-200/80 dark:border-slate-700/80 hover:border-rose-200 dark:hover:border-rose-900 text-slate-600 dark:text-slate-300 hover:text-rose-600 dark:hover:text-rose-400 text-xs font-medium rounded-xl flex items-center justify-center gap-1.5 transition-colors cursor-pointer"
            >
              <Power class="w-3.5 h-3.5 text-rose-500" />
              <span>Thoát hẳn ứng dụng (Tắt chạy ngầm)</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>