# 📝 TickTask - Personal Task Manager (Tauri v2 + Vue 3 + Rust SQLite)

Ứng dụng Desktop quản lý công việc cá nhân phong cách **TickTick**, được xây dựng trên nền tảng **Tauri v2**, **Vue 3 (TypeScript)**, **Tailwind CSS** và **Rust SQLite Local-first (Repository Pattern)**.

---

## 🌟 Điểm nổi bật & Tính năng chính

- **Kiến trúc Local-first (Offline-first & Sẵn sàng Supabase):**
  - Tầng Backend Rust lưu trữ dữ liệu cục bộ bằng **SQLite** (`sqlx` runtime query + migration tự động tại `~/.local/share/rust-task-mgr/tasks.db`).
  - Định danh bằng **UUID v7**, mốc thời gian Unix timestamp tính bằng mili-giây (`created_at`, `updated_at`) phục vụ thuật toán Last-Write-Wins (LWW).
  - Sử dụng cơ chế **Xóa mềm (Soft Delete - `deleted_at`)** và trạng thái đồng bộ (`sync_status: 'pending' | 'synced'`) sẵn sàng kết nối và đồng bộ 2 chiều lên Supabase (PostgreSQL).
- **Giao diện 3 Cột Hiện đại (Chuẩn UI/UX TickTick):**
  - **Cột 1 — Smart Sidebar (rộng ~220px, có thể thu gọn):**
    - Bộ lọc thông minh kèm số đếm (badge) theo thời gian thực: *Tất cả công việc*, *Hôm nay & Quá hạn*, *Ưu tiên cao*, *Đã hoàn thành*.
    - Nút chuyển giao diện **Sáng / Tối (Dark Mode Slate/Zinc dịu mắt)**.
    - Trạng thái SQLite Local-first ở chân sidebar (*Chờ đồng bộ: X task*).
  - **Cột 2 — Khu vực làm việc chính (Linh hoạt List View & Kanban View):**
    - **Quick Add Input (Đặc trưng TickTick):** Thanh nhập liệu nổi bật phía trên, gõ tên việc nhấn `Enter` để lưu ngay; tích hợp nút chọn nhanh **Độ ưu tiên (Cờ màu: Xám / Xanh / Vàng / Đỏ)** và **Ngày hạn (Hôm nay / Ngày mai / Chọn ngày)**.
    - **Chế độ List View:** Task hiển thị dạng dòng với checkbox tròn đổi màu viền theo mức độ ưu tiên; phân nhóm thông minh theo `Đang thực hiện`, `Cần làm` và mục `Đã hoàn thành` có thể thu gọn/mở rộng.
    - **Chế độ Kanban View:** 3 cột đều nhau (`Cần làm`, `Đang làm`, `Hoàn thành`) hỗ trợ kéo-thả **HTML5 Drag & Drop** mượt mà, tự động cập nhật trạng thái xuống SQLite.
    - **Header Bar:** Tiêu đề danh mục, ô tìm kiếm nhanh (Search) và nút chuyển chế độ xem 1-click.
  - **Cột 3 — Task Detail Drawer (rộng ~320px):**
    - Mở khi click vào bất kỳ task nào; cho phép chỉnh sửa trực tiếp với cơ chế **Auto-save (debounce 400ms / on blur)**.
    - Sửa tiêu đề, mô tả/ghi chú chi tiết, ngày đến hạn, độ ưu tiên, trạng thái, metadata thời gian và nút xóa mềm task.

---

## 🏗️ Kiến trúc thư mục (Architecture)

```text
my-todo-rust/
├── package.json                 # Scripts và frontend dependencies
├── vite.config.ts               # Cấu hình Vite cho Tauri v2
├── tsconfig.json                # TypeScript compiler options
├── tailwind.config.js           # Cấu hình Tailwind CSS & Dark mode
├── index.html                   # HTML entrypoint nạp font Inter & viewport
├── src/                         # FRONTEND (Vue 3 + TS + Tailwind)
│   ├── main.ts                  # Khởi tạo Vue app và Pinia store
│   ├── App.vue                  # Bố cục 3-Pane Layout
│   ├── style.css                # Tailwind directives & custom scrollbars
│   ├── types/
│   │   └── task.ts              # Interface Task, TaskStatus, Priority, FilterType
│   ├── stores/
│   │   └── taskStore.ts         # Pinia store giao tiếp backend qua invoke()
│   └── components/
│       ├── Sidebar.vue          # Cột 1: Smart Sidebar & Sync status
│       ├── QuickAddBar.vue      # Thanh nhập việc nhanh đặc trưng TickTick
│       ├── TaskListView.vue     # Cột 2: Giao diện danh sách phân nhóm
│       ├── TaskKanbanView.vue   # Cột 2: Bảng Kanban kéo thả Drag & Drop
│       └── TaskDetailPanel.vue  # Cột 3: Drawer chi tiết công việc & Auto-save
└── src-tauri/                   # BACKEND (Tauri v2 + Rust)
    ├── Cargo.toml               # Tauri v2, sqlx, tokio, uuid, serde
    ├── tauri.conf.json          # Cấu hình cửa sổ và quyền ứng dụng
    ├── build.rs                 # Script build Tauri
    ├── capabilities/
    │   └── default.json         # Phân quyền core permissions
    ├── migrations/
    │   └── 0001_create_tasks.sql # Schema bảng tasks SQLite
    └── src/
        ├── main.rs              # Entrypoint binary
        ├── lib.rs               # Setup tauri builder & state manage
        ├── commands.rs          # Các hàm #[tauri::command] gọi TaskRepository
        ├── domain/
        │   ├── mod.rs
        │   └── task.rs          # Entity Task, TaskStatus, Priority
        └── storage/
            ├── mod.rs           # TaskRepository trait
            └── sqlite.rs        # SqliteTaskRepo (WAL mode, auto-migration)
```

---

## 📦 Yêu cầu hệ thống & Cài đặt

### 1. Cài đặt các gói thư viện hệ thống cần thiết (Ubuntu 24.04 / Debian)
Tauri v2 trên Linux yêu cầu WebKitGTK và một số thư viện phát triển giao diện:

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libxdo-dev \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev
```

### 2. Cài đặt Node dependencies (pnpm)
```bash
pnpm install
```

### 3. Chạy ứng dụng ở môi trường phát triển (Dev)
```bash
# Chạy ứng dụng Tauri v2 Desktop (kèm Hot Reload frontend & backend)
pnpm tauri dev

# Hoặc chạy thử nhanh giao diện trên trình duyệt (có sẵn Mock Data Fallback)
pnpm dev
```

### 4. Đóng gói & Kiểm tra (Production Build)
```bash
pnpm build
pnpm tauri build
```
