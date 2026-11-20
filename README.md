# 📝 TickTask - Personal Task & Calendar Manager

> Ứng dụng Desktop quản lý công việc và xếp lịch cá nhân phong cách **TickTick**, xây dựng trên nền tảng **Tauri v2**, **Vue 3 (TypeScript)**, **Tailwind CSS**, kiến trúc **Local-first SQLite** và **Đồng bộ hai chiều Supabase Cloud**.

---

## 🌟 Điểm nổi bật & Tính năng chính

### 1. 📅 Màn hình Lịch (Calendar View) & Time-blocking (Chuẩn TickTick)
- **4 Chế độ hiển thị linh hoạt:** `Ngày (Day)`, `Tuần (Week)`, `Tháng (Month)` và `Lịch trình (Agenda)`.
- **Lưới thời gian 24h (Time-blocking Grid):** Trực quan hóa các block thời gian trong ngày với đường kẻ giờ hiện tại (Current Time Indicator) cập nhật theo thời gian thực.
- **Kéo-thả xếp lịch (Drag & Drop):** Kéo các công việc chưa xếp lịch từ thanh bên **Unscheduled Tasks Drawer** và thả vào khung giờ tương ứng để tự động cập nhật `start_time` và `end_time`.
- **Thu gọn khung giờ đêm (Night hours):** Tùy chọn thu gọn khoảng thời gian từ `00:00 - 06:00` để tập trung vào giờ làm việc hiệu quả.
- **Điều hướng nhanh:** Di chuyển tới/lui giữa các tuần, tháng và nút 1-click quay về "Hôm nay".

### 2. 📋 Quản lý Công việc & Bảng Kanban
- **Navigation Rail ngoài cùng bên trái (54px):** Chuyển đổi nhanh giữa các màn hình **Tasks**, **Calendar**, **Search**, quản lý tài khoản/đồng bộ và nút chuyển đổi giao diện Sáng/Tối.
- **Smart Sidebar (~220px, có thể thu gọn):** Bộ lọc thông minh kèm số đếm thời gian thực: *Tất cả công việc*, *Hôm nay & Quá hạn*, *Ưu tiên cao*, *Đã hoàn thành*.
- **Quick Add Bar:** Thanh tạo việc nhanh đặc trưng TickTick, gõ tên việc nhấn `Enter`, chọn nhanh độ ưu tiên (Cờ 4 màu) và ngày hết hạn.
- **Linh hoạt 2 chế độ hiển thị:**
  - **Danh sách (List View):** Phân nhóm thông minh theo trạng thái công việc (`Đang làm`, `Cần làm`, `Đã xong`).
  - **Bảng Kanban (Kanban View):** 3 cột kéo-thả HTML5 Drag & Drop mượt mà, lưu trạng thái tức thì xuống SQLite.
- **Task Detail Drawer:** Tự động lưu (Auto-save debounce 400ms), hỗ trợ chỉnh sửa chi tiết tiêu đề, ghi chú, độ ưu tiên, ngày hết hạn và khung giờ bắt đầu / kết thúc.

### 3. 🔍 Tìm kiếm toàn cục (Spotlight / Command Palette - `Ctrl + K`)
- Mở nhanh ở bất kỳ màn hình nào bằng tổ hợp phím **`Ctrl + K`** (Windows/Linux) hoặc `Cmd + K` (macOS), hoặc bấm vào biểu tượng kính lúp trên Navigation Rail.
- **Tìm kiếm tiếng Việt không dấu thông minh:** Người dùng gõ `"cong viec"` vẫn tìm thấy `"Công việc A"`.
- Bộ lọc nhanh theo trạng thái (`Tất cả`, `Đang chờ`, `Đã hoàn thành`) và tự động ưu tiên kết quả chưa hoàn thành lên trên.

### 4. ⚡ Kiến trúc Local-first & Đồng bộ Supabase (Offline-first)
- **SQLite Cục bộ luôn là Source of Truth:** Mọi thao tác tạo, sửa, xóa, kéo-thả ghi trực tiếp xuống SQLite cục bộ với độ trễ 0ms, hoạt động ổn định ngay cả khi không có kết nối Internet.
- **Chế độ Khách (Guest mode):** Sử dụng đầy đủ chức năng ngay từ lần đầu mở app mà không bắt buộc đăng nhập.
- **Tự động chuyển đổi dữ liệu (Auto-migration):** Khi đăng nhập hoặc đăng ký tài khoản, các công việc đã tạo ở chế độ khách sẽ tự động được gán vào tài khoản người dùng mới.
- **Đồng bộ ngầm hai chiều (Push & Pull):**
  - Kênh điều phối ngầm gom các thay đổi (Debounce 1.5s) và chu kỳ định kỳ 60s.
  - Tự động đẩy task `pending` lên Supabase PostgREST và kéo các thay đổi mới về SQLite bằng giải thuật **Last-Write-Wins (LWW)**.
  - Bắt sự kiện `tasks-synced` để tự động làm mới giao diện Vue 3 theo thời gian thực.
- **Bảo mật tuyệt đối với Supabase Auth & RLS:** Xác thực bằng GoTrue API, dữ liệu người dùng được cô lập triệt để bằng PostgreSQL Row Level Security (`auth.uid() = user_id`).
- **Nhúng cấu hình lúc biên dịch (Compile-time embedding):** Tự động nhúng `SUPABASE_URL` và `SUPABASE_ANON_KEY` qua `build.rs` và `option_env!`, cho phép bản cài đặt `.deb` / `.AppImage` hoạt động ở mọi thư mục hệ thống mà không phụ thuộc file `.env`.

---

## 🏗️ Cấu trúc Thư mục Dự án

```text
my-todo-rust/
├── Makefile                     # Các lệnh tiện ích tự động hóa toàn diện
├── package.json                 # Scripts và dependencies phía frontend
├── vite.config.ts               # Cấu hình Vite bundler cho Tauri v2
├── tsconfig.json                # Cấu hình TypeScript
├── tailwind.config.js           # Cấu hình Tailwind CSS & Dark mode
├── index.html                   # HTML entrypoint
├── supabase/
│   └── schema.sql               # Bảng tasks, RLS policies & Permissions trên Supabase
├── src/                         # FRONTEND (Vue 3 + TypeScript + Tailwind)
│   ├── main.ts                  # Khởi tạo Vue app và Pinia store
│   ├── App.vue                  # Layout chính & tích hợp các Modal toàn cục
│   ├── types/
│   │   └── task.ts              # Kiểu Task, UserInfo, SyncStatusState, ViewMode
│   ├── stores/
│   │   └── taskStore.ts         # Pinia store điều phối dữ liệu, auth và sync
│   └── components/
│       ├── NavigationRail.vue   # Thanh điều hướng ngoài cùng bên trái (54px)
│       ├── Sidebar.vue          # Smart Sidebar bộ lọc thông minh (220px)
│       ├── QuickAddBar.vue      # Thanh nhập công việc nhanh đặc trưng TickTick
│       ├── TaskListView.vue     # Chế độ xem danh sách phân nhóm
│       ├── TaskKanbanView.vue   # Chế độ xem bảng Kanban kéo thả
│       ├── CalendarView.vue     # Màn hình Lịch biểu & Time-blocking Grid
│       ├── UnscheduledDrawer.vue# Danh sách công việc chưa xếp lịch để kéo thả
│       ├── TaskDetailPanel.vue  # Cột phải chi tiết công việc & Auto-save
│       ├── SearchModal.vue      # Spotlight Command Palette tìm kiếm toàn cục (Ctrl+K)
│       └── AuthModal.vue        # Modal đăng nhập, đăng ký và bảng điều khiển Sync
└── src-tauri/                   # BACKEND (Tauri v2 + Rust)
    ├── Cargo.toml               # Cấu hình dependencies: sqlx, tokio, reqwest, dotenvy
    ├── tauri.conf.json          # Cấu hình cửa sổ, icon và đóng gói deb/appimage
    ├── build.rs                 # Script nạp .env và nhúng biến môi trường lúc compile
    ├── migrations/
    │   ├── 0001_create_tasks.sql       # Khởi tạo bảng tasks SQLite ban đầu
    │   ├── 0002_add_calendar_time.sql   # Thêm start_time, end_time cho Calendar
    │   └── 0003_add_auth_and_sync.sql   # Thêm user_id, auth_session, sync_meta
    └── src/
        ├── main.rs              # Entrypoint binary Tauri
        ├── lib.rs               # Setup Tauri builder, state injection và sync worker
        ├── commands.rs          # Các hàm #[tauri::command] IPC giao tiếp với frontend
        ├── domain/
        │   ├── task.rs          # Struct Task, TaskStatus, Priority
        │   └── auth.rs          # Struct AuthSession, UserInfo, SyncStatusState
        ├── storage/
        │   ├── mod.rs           # TaskRepository trait
        │   └── sqlite.rs        # Triển khai SQLite (WAL mode, Last-Write-Wins, Multi-user)
        └── sync/
            ├── auth.rs          # GoTrue client (Login, Register, Auto-refresh token)
            └── sync_engine.rs   # Sync Engine hai chiều (Debounce channel, Push, Pull)
```

---

## 📦 Yêu cầu Hệ thống & Cài đặt

### 1. Cài đặt thư viện hệ thống (Ubuntu 24.04 / Debian)
```bash
make install-sys-deps
```
*(Lệnh trên sẽ tự động cài đặt `libglib2.0-dev`, `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libssl-dev`, `pkg-config`, `build-essential`, ...)*

### 2. Cài đặt Node dependencies
```bash
make install
# hoặc: pnpm install
```

### 3. Cấu hình biến môi trường Supabase
1. Khởi tạo file cấu hình `.env` cho backend:
   ```bash
   make env-init
   # hoặc: cp src-tauri/.env.example src-tauri/.env
   ```
2. Mở file `src-tauri/.env` và cập nhật thông tin dự án Supabase của bạn:
   ```env
   SUPABASE_URL=https://your-project-id.supabase.co
   SUPABASE_ANON_KEY=your-public-anon-key
   ```
3. Chạy script SQL tại [supabase/schema.sql](supabase/schema.sql) trên **Supabase SQL Editor** để khởi tạo bảng và chính sách bảo mật RLS.

---

## 🚀 Khởi chạy Phát triển (Development)

```bash
# Khởi chạy ứng dụng Desktop Tauri v2 (kèm Hot-reload frontend & backend)
make dev
# hoặc: pnpm tauri dev

# Chạy thử nhanh giao diện trên trình duyệt (có sẵn Mock Data Fallback)
make dev-web
# hoặc: pnpm dev
```

---

## 🛠️ Đóng gói & Cài đặt (Release Packaging)

Ứng dụng hỗ trợ đóng gói chuẩn cho Linux thành các gói `.deb` (cài đặt hệ thống) và `.AppImage` (chạy độc lập):

```bash
# Đóng gói hoàn chỉnh cả 2 định dạng (.deb và .AppImage)
make build
# hoặc: pnpm tauri build

# Đóng gói riêng file .deb
make build-deb

# Đóng gói riêng file .AppImage
make build-appimage

# Cài đặt file .deb vừa build vào máy Ubuntu của bạn
make install-deb

# Khởi chạy trực tiếp file .AppImage vừa build
make run-appimage
```

*Các gói cài đặt sau khi đóng gói sẽ nằm tại:*
- `.deb`: `src-tauri/target/release/bundle/deb/TickTask_0.1.0_amd64.deb`
- `.AppImage`: `src-tauri/target/release/bundle/appimage/TickTask_0.1.0_amd64.AppImage`

### Đóng gói cho macOS (MacBook chip Apple Silicon M-series & Intel):
> **Lưu ý:** Do yêu cầu về Apple SDK, Xcode Command Line Tools và macOS GUI framework, việc đóng gói ứng dụng macOS (`.dmg` và `.app`) cần được thực hiện trực tiếp trên máy Mac (hoặc thông qua GitHub Actions runner `macos-latest`):

```bash
# 1. Cài đặt Xcode Command Line Tools trên Mac (chỉ làm lần đầu)
xcode-select --install

# 2. Đóng gói chuẩn theo chip của máy Mac hiện tại (.dmg & .app)
make build-mac
# hoặc: pnpm tauri build --bundles dmg,app

# 3. Đóng gói Universal Binary (chạy mượt trên CẢ chip Apple Silicon M1/M2/M3/M4 và Intel)
rustup target add aarch64-apple-darwin x86_64-apple-darwin
make build-mac-universal
# hoặc: pnpm tauri build --target universal-apple-darwin
```

*File sau khi đóng gói trên macOS sẽ nằm tại:*
- File cài đặt kéo-thả `.dmg`: `src-tauri/target/release/bundle/dmg/TickTask_0.1.0_*.dmg`
- Ứng dụng độc lập `.app`: `src-tauri/target/release/bundle/macos/TickTask.app`

---

## 📋 Danh sách Lệnh Makefile (Tra cứu nhanh)

Dự án cung cấp `Makefile` với đầy đủ các lệnh tiện ích tự động hóa:

| Lệnh | Mô tả |
| :--- | :--- |
| `make help` | Hiển thị bảng trợ giúp danh sách toàn bộ các lệnh có sẵn |
| **Cài đặt & Thiết lập** | |
| `make install` | Cài đặt các thư viện Node.js (`pnpm install`) |
| `make install-sys-deps` | Cài đặt toàn bộ thư viện hệ thống Linux cần thiết cho Tauri v2 |
| `make env-init` | Khởi tạo file `src-tauri/.env` từ file mẫu `.env.example` |
| `make setup` | Chạy thiết lập ban đầu hoàn chỉnh |
| **Môi trường Phát triển** | |
| `make dev` | Chạy ứng dụng Desktop Tauri dev mode (hot-reload) |
| `make dev-web` | Chạy frontend web dev server trên trình duyệt (Vite) |
| `make preview` | Chạy thử bản build frontend cục bộ |
| **Đóng gói & Phân phối** | |
| `make build` | Đóng gói bản Release hoàn chỉnh (`.deb` và `.AppImage`) |
| `make build-deb` | Đóng gói riêng bản cài đặt Debian (`.deb`) |
| `make build-appimage` | Đóng gói riêng bản thực thi di động (`.AppImage`) |
| `make build-debug` | Đóng gói ứng dụng Desktop ở chế độ Debug |
| `make build-web` | Biên dịch tài sản frontend (vue-tsc + vite build) |
| `make build-rust` | Biên dịch mã nguồn Rust backend ở chế độ release |
| `make install-deb` | Cài đặt gói `.deb` vừa tạo vào hệ thống bằng `dpkg` |
| `make run-appimage` | Cấp quyền thực thi và chạy trực tiếp file `.AppImage` |
| **Kiểm tra & Chất lượng mã** | |
| `make check` | Chạy kiểm tra toàn diện (typecheck, fmt-check, lint, test) |
| `make check-release` | Kiểm tra biên dịch Rust backend ở chế độ Release với biến nhúng |
| `make typecheck` | Kiểm tra kiểu dữ liệu TypeScript (`vue-tsc --noEmit`) |
| `make test` | Chạy toàn bộ Unit & Integration tests phía Rust backend |
| `make lint` | Chạy linter Clippy trên Rust backend |
| `make fmt` | Tự động định dạng mã nguồn Rust (`cargo fmt`) |
| `make fmt-check` | Kiểm tra định dạng mã nguồn Rust mà không thay đổi file |
| **Cơ sở dữ liệu & Đồng bộ** | |
| `make db-path` | Xem đường dẫn file SQLite cục bộ (`tasks.db`) |
| `make db-browse` | Mở trực tiếp cơ sở dữ liệu SQLite bằng `sqlite3` CLI |
| `make db-clean` | Xóa cơ sở dữ liệu SQLite cục bộ để đặt lại trạng thái ban đầu |
| `make supabase-schema` | Hiển thị đường dẫn và hướng dẫn chạy script Supabase |
| **Dọn dẹp** | |
| `make clean` | Dọn dẹp thư mục build (`dist`, `src-tauri/target`) |
| `make clean-all` | Dọn dẹp sâu cả thư mục build và `node_modules` |

---

## 🔒 Bản quyền & Giấy phép
Dự án được phân phối dưới giấy phép mã nguồn mở MIT License. Tự do sử dụng, chỉnh sửa và phát triển.
*Các gói cài đặt sau khi đóng gói sẽ nằm tại:*
- `.deb`: `src-tauri/target/release/bundle/deb/TickTask_0.1.0_amd64.deb`
- `.AppImage`: `src-tauri/target/release/bundle/appimage/TickTask_0.1.0_amd64.AppImage`

### Đóng gói cho macOS (MacBook chip Apple Silicon M-series & Intel):
> **Lưu ý:** Do yêu cầu về Apple SDK, Xcode Command Line Tools và macOS GUI framework, việc đóng gói ứng dụng macOS (`.dmg` và `.app`) cần được thực hiện trực tiếp trên máy Mac (hoặc thông qua GitHub Actions runner `macos-latest`):

```bash
# 1. Cài đặt Xcode Command Line Tools trên Mac (chỉ làm lần đầu)
xcode-select --install

# 2. Đóng gói chuẩn theo chip của máy Mac hiện tại (.dmg & .app)
make build-mac
# hoặc: pnpm tauri build --bundles dmg,app

# 3. Đóng gói Universal Binary (chạy mượt trên CẢ chip Apple Silicon M1/M2/M3/M4 và Intel)
rustup target add aarch64-apple-darwin x86_64-apple-darwin
make build-mac-universal
# hoặc: pnpm tauri build --target universal-apple-darwin
```

*File sau khi đóng gói trên macOS sẽ nằm tại:*
- File cài đặt kéo-thả `.dmg`: `src-tauri/target/release/bundle/dmg/TickTask_0.1.0_*.dmg`
- Ứng dụng độc lập `.app`: `src-tauri/target/release/bundle/macos/TickTask.app`

---

## 📋 Danh sách Lệnh Makefile (Tra cứu nhanh)

Dự án cung cấp `Makefile` với đầy đủ các lệnh tiện ích tự động hóa:

| Lệnh | Mô tả |
| :--- | :--- |
| `make help` | Hiển thị bảng trợ giúp danh sách toàn bộ các lệnh có sẵn |
| **Cài đặt & Thiết lập** | |
| `make install` | Cài đặt các thư viện Node.js (`pnpm install`) |
| `make install-sys-deps` | Cài đặt toàn bộ thư viện hệ thống Linux cần thiết cho Tauri v2 |
| `make env-init` | Khởi tạo file `src-tauri/.env` từ file mẫu `.env.example` |
| `make setup` | Chạy thiết lập ban đầu hoàn chỉnh |
| **Môi trường Phát triển** | |
| `make dev` | Chạy ứng dụng Desktop Tauri dev mode (hot-reload) |
| `make dev-web` | Chạy frontend web dev server trên trình duyệt (Vite) |
| `make preview` | Chạy thử bản build frontend cục bộ |
| **Đóng gói & Phân phối** | |
| `make build` | Đóng gói bản Release hoàn chỉnh (`.deb` và `.AppImage`) |
| `make build-deb` | Đóng gói riêng bản cài đặt Debian (`.deb`) |
| `make build-appimage` | Đóng gói riêng bản thực thi di động (`.AppImage`) |
| `make build-debug` | Đóng gói ứng dụng Desktop ở chế độ Debug |
| `make build-web` | Biên dịch tài sản frontend (vue-tsc + vite build) |
| `make build-rust` | Biên dịch mã nguồn Rust backend ở chế độ release |
| `make install-deb` | Cài đặt gói `.deb` vừa tạo vào hệ thống bằng `dpkg` |
| `make run-appimage` | Cấp quyền thực thi và chạy trực tiếp file `.AppImage` |
| **Kiểm tra & Chất lượng mã** | |
| `make check` | Chạy kiểm tra toàn diện (typecheck, fmt-check, lint, test) |
| `make check-release` | Kiểm tra biên dịch Rust backend ở chế độ Release với biến nhúng |
| `make typecheck` | Kiểm tra kiểu dữ liệu TypeScript (`vue-tsc --noEmit`) |
| `make test` | Chạy toàn bộ Unit & Integration tests phía Rust backend |
| `make lint` | Chạy linter Clippy trên Rust backend |
| `make fmt` | Tự động định dạng mã nguồn Rust (`cargo fmt`) |
| `make fmt-check` | Kiểm tra định dạng mã nguồn Rust mà không thay đổi file |
| **Cơ sở dữ liệu & Đồng bộ** | |
| `make db-path` | Xem đường dẫn file SQLite cục bộ (`tasks.db`) |
| `make db-browse` | Mở trực tiếp cơ sở dữ liệu SQLite bằng `sqlite3` CLI |
| `make db-clean` | Xóa cơ sở dữ liệu SQLite cục bộ để đặt lại trạng thái ban đầu |
| `make supabase-schema` | Hiển thị đường dẫn và hướng dẫn chạy script Supabase |
| **Dọn dẹp** | |
| `make clean` | Dọn dẹp thư mục build (`dist`, `src-tauri/target`) |
| `make clean-all` | Dọn dẹp sâu cả thư mục build và `node_modules` |

---

## 🔒 Bản quyền & Giấy phép
Dự án được phân phối dưới giấy phép mã nguồn mở MIT License. Tự do sử dụng, chỉnh sửa và phát triển.
