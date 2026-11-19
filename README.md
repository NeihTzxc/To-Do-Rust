# 📝 Personal Task Manager (Rust Desktop)

Ứng dụng Desktop quản lý công việc cá nhân được viết hoàn toàn bằng ngôn ngữ **Rust**, chạy trên hệ điều hành **Ubuntu 24.04** (và các bản phân phối Linux khác) theo kiến trúc **Local-first** và **Repository Pattern**, được thiết kế sẵn sàng để mở rộng đồng bộ hai chiều lên **Supabase (PostgreSQL)**.

---

## 🌟 Điểm nổi bật & Tính năng chính

- **Kiến trúc Local-first (Offline-first):**
  - Dữ liệu được lưu trữ cục bộ tại máy tính người dùng bằng **SQLite** (`sqlx` runtime) theo chuẩn đường dẫn **XDG Linux** (`~/.local/share/rust-task-mgr/tasks.db`).
  - Tự động tạo thư mục và tự chạy migration (`sqlx::migrate!()`) ngay khi ứng dụng khởi chạy.
- **Sẵn sàng đồng bộ Cloud (Supabase / PostgreSQL):**
  - Schema được trang bị sẵn UUID v7, mốc thời gian Unix timestamp tính bằng mili-giây (`created_at`, `updated_at`) phục vụ thuật toán **Last-Write-Wins (LWW)**.
  - Sử dụng cơ chế **Xóa mềm (Soft Delete - `deleted_at`)** và trạng thái đồng bộ (`sync_status: 'pending' | 'synced'`) giúp việc đồng bộ 2 chiều không bị mất dữ liệu.
- **Chế độ xem linh hoạt (Dual View Modes):**
  - 📊 **Bảng Kanban (3 cột):** Cần làm (`todo`), Đang làm (`in_progress`), Hoàn thành (`done`). Hỗ trợ **Kéo và Thả (Drag & Drop)** native của `egui` cùng các phím mũi tên chuyển cột nhanh 1-click (`←` / `→`).
  - 📋 **Danh sách truyền thống (List View):** Lọc theo *Tất cả*, *Đang làm*, *Đã xong*, checkbox đánh dấu hoàn thành (tiêu đề gạch ngang), badge mức độ ưu tiên.
- **Nhập việc nhanh chóng:**
  - Ô nhập công việc hỗ trợ phím **Enter** để thêm tức thì, chọn mức độ ưu tiên (*Low / Medium / High*) và nút mở rộng nhập mô tả chi tiết.
- **Tìm kiếm & Bộ lọc tức thời:**
  - Lọc công việc theo từ khóa tiêu đề hoặc mô tả trong thời gian thực.
- **Hiển thị tiếng Việt hoàn hảo:**
  - Tự động nạp font hệ thống Ubuntu (`Noto Sans`, `DejaVu Sans`) vào `egui::FontDefinitions`, khắc phục triệt để lỗi ô vuông font (tofu).
- **Giao diện mượt mà & Non-blocking:**
  - Render đồ họa thuần GPU/CPU bằng `eframe` / `egui` (không dùng WebView nặng nề).
  - Tầng dữ liệu chạy trên luồng nền **Tokio Async Runtime**, giao tiếp với UI thông qua channel (`tokio::sync::mpsc` và `std::sync::mpsc`), không bao giờ gây đơ hay drop frame khi đọc/ghi đĩa.
  - Thiết kế co giãn linh hoạt (**Responsive Layout**), tự động ngắt dòng khi thu hẹp cửa sổ.

---

## 🏗️ Kiến trúc thư mục (Architecture)

```text
my-todo-rust/
├── Cargo.toml
├── migrations/
│   └── 0001_create_tasks.sql    # Schema bảng tasks hỗ trợ Soft-delete & Sync
├── tests/
│   └── repository_test.rs       # Bộ test tự động CRUD, Soft Delete & Sync Status
└── src/
    ├── lib.rs                   # Expose các module cho testing & integration
    ├── main.rs                  # Khởi tạo Tokio đa luồng, chạy migration và eframe window loop
    ├── domain/
    │   ├── mod.rs
    │   └── task.rs              # Struct Task, TaskStatus, Priority (UUID v7, Unix ms)
    ├── storage/
    │   ├── mod.rs               # Async trait TaskRepository
    │   └── sqlite.rs            # SqliteTaskRepo (quản lý pool, auto-migration, XDG path)
    └── ui/
        ├── mod.rs
        └── app.rs               # UI State Management, Kanban Board DnD & List View
```

---

## 📊 Thiết kế Schema Database (`tasks`)

```sql
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY NOT NULL,              -- Chuỗi UUID v7 tự sắp xếp theo thời gian
    title TEXT NOT NULL,                       -- Tiêu đề công việc
    description TEXT,                          -- Mô tả chi tiết (nullable)
    status TEXT NOT NULL DEFAULT 'todo',       -- 'todo', 'in_progress', 'done'
    priority INTEGER NOT NULL DEFAULT 2,       -- 1: Low, 2: Medium, 3: High
    due_date INTEGER,                          -- Unix timestamp hạn chót (nullable)
    created_at INTEGER NOT NULL,               -- Unix timestamp ms
    updated_at INTEGER NOT NULL,               -- Unix timestamp ms (LWW sync)
    deleted_at INTEGER,                        -- Unix timestamp ms (Soft Delete)
    sync_status TEXT NOT NULL DEFAULT 'pending'-- 'pending' hoặc 'synced'
);

CREATE INDEX IF NOT EXISTS idx_tasks_deleted_at ON tasks(deleted_at);
CREATE INDEX IF NOT EXISTS idx_tasks_updated_at ON tasks(updated_at);
CREATE INDEX IF NOT EXISTS idx_tasks_sync_status ON tasks(sync_status);
```

---

## 📦 Yêu cầu môi trường & Cài đặt thư viện hệ thống

Ứng dụng yêu cầu **Rust toolchain** (1.75 trở lên) và các gói thư viện phát triển đồ họa trên Ubuntu:

```bash
# Cài đặt Rust nếu chưa có
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# Cài đặt các gói thư viện hệ thống trên Ubuntu/Debian
sudo apt update
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libxcb1-dev \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libxkbcommon-dev \
    libxkbcommon-x11-dev \
    libfontconfig1-dev
```

---

## 🚀 Hướng dẫn khởi chạy

### 1. Chạy ở chế độ phát triển (Development)
```bash
cargo run
```

### 2. Biên dịch bản chính thức (Release)
Bản release được tối ưu hóa hiệu năng và dung lượng:
```bash
cargo build --release
./target/release/rust-task-mgr
```

### 3. Chạy bộ kiểm thử tự động (Tests)
```bash
cargo test
```

---

## ⚙️ Cấu hình nâng cao

| Biến môi trường | Mặc định | Mô tả |
| :--- | :--- | :--- |
| `TASK_MGR_DB_PATH` | `~/.local/share/rust-task-mgr/tasks.db` | Ghi đè đường dẫn file SQLite (hữu ích khi test hoặc backup dữ liệu). |

---

## 🗺️ Lộ trình phát triển (Roadmap)

- [x] Khởi tạo kiến trúc phân tầng: Domain, Storage (Repository Pattern), UI.
- [x] Lưu trữ cục bộ SQLite với auto-migration và chuẩn XDG.
- [x] Chế độ xem Danh sách (List View) và Bảng Kanban (Kanban Board 3 cột).
- [x] Kéo & thả (Drag & Drop) và phím chuyển cột nhanh 1-click.
- [x] Hỗ trợ hiển thị tiếng Việt có dấu qua font Ubuntu.
- [ ] Tích hợp Supabase Client: Tự động phát hiện mạng và đồng bộ dữ liệu hai chiều khi online.
- [ ] Thông báo Desktop (Desktop Notification) khi đến hạn `due_date`.
- [ ] Gán nhãn/Tag và phân loại theo dự án.
