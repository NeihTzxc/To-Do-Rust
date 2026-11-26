# Kế hoạch triển khai đăng nhập Google và Apple

Tài liệu này dành cho lần phát triển tiếp theo của TickTask desktop (Tauri 2, Vue 3, Rust, Supabase Auth). Mục tiêu là thêm hai cách đăng nhập mà vẫn dùng phiên Supabase hiện có để đồng bộ task. Phạm vi là ứng dụng desktop; web preview hiện chỉ dùng dữ liệu giả và cần luồng callback riêng nếu sau này hỗ trợ đăng nhập thật trên web.

## Trạng thái hiện tại

- `src/components/AuthModal.vue` có biểu mẫu email và mật khẩu; `src/stores/taskStore.ts` gọi các lệnh `login` và `register` của Tauri.
- `src-tauri/src/sync/auth.rs` gọi Supabase GoTrue, lưu `access_token`, `refresh_token` và hạn phiên qua repository. `src-tauri/src/storage/sqlite.rs` lưu phiên trong bảng `auth_session` và gán task khách cho người dùng sau khi đăng nhập.
- `src-tauri/src/commands.rs` chạy một chu kỳ đồng bộ sau đăng nhập. `src-tauri/src/lib.rs` đã dùng plugin single instance để mở lại cửa sổ, nhưng chưa nhận callback đăng nhập.
- Chưa có cấu hình Google/Apple provider, deep link, mở URL trong trình duyệt hệ thống hoặc trao đổi mã OAuth lấy phiên.

## Quyết định thiết kế đề xuất

Dùng **Supabase Auth OAuth với PKCE** cho cả Google và Apple. Mở trang đăng nhập trong trình duyệt hệ thống; Supabase chuyển người dùng về một deep link của TickTask, ví dụ `ticktask://auth/callback`. Chỉ mã xác thực đi qua deep link. Rust kiểm tra callback, trao đổi mã cùng PKCE verifier lấy phiên và lưu bằng `save_session` hiện có. Sau đó chạy đồng bộ, cập nhật `currentUser` và đóng màn hình đăng nhập.

```text
TickTask → trình duyệt → Google hoặc Apple → Supabase Auth
         ← ticktask://auth/callback?code=... ← Supabase Auth
Rust kiểm tra callback → đổi code lấy phiên → lưu SQLite → đồng bộ task
```

Để tránh hai nơi giữ phiên không đồng bộ, **Rust/SQLite là nơi lưu phiên chính**. Nếu chọn dùng `supabase-js` để tạo URL và xử lý PKCE thay cho Rust, cần thiết kế rõ cách chuyển phiên sang Rust và tránh giữ thêm phiên lâu dài trong webview. Chốt một cách trước khi code.

Có hai URL callback khác nhau:

1. **Google/Apple → Supabase:** `https://<project-ref>.supabase.co/auth/v1/callback`. Đăng ký URL này ở Google Cloud hoặc Apple Developer.
2. **Supabase → TickTask:** `ticktask://auth/callback` (tên scheme là đề xuất). Thêm URL này vào allow list Redirect URLs của Supabase và đăng ký scheme trong Tauri.

Nguồn: [Supabase Google](https://supabase.com/docs/guides/auth/social-sign-in/auth-google), [Supabase Apple](https://supabase.com/docs/guides/auth/social-sign-in/auth-apple), [Supabase Redirect URLs](https://supabase.com/docs/guides/auth/redirect-urls), [Tauri deep links](https://v2.tauri.app/plugin/deep-linking/).

## Chuẩn bị cấu hình

### Supabase

1. Xác nhận dự án Supabase đang dùng trong `SUPABASE_URL` và `SUPABASE_ANON_KEY`.
2. Thêm `ticktask://auth/callback` vào Auth Redirect URLs. Dùng URL chính xác cho từng môi trường nếu dev và production dùng scheme khác nhau.
3. Bật provider Google và Apple trong Authentication → Providers sau khi có thông tin từ hai hệ thống bên dưới.
4. Kiểm tra lại chính sách RLS trong `supabase/schema.sql`; task tiếp tục được gắn với `auth.uid()` của phiên Supabase.

### Google Cloud

1. Tạo hoặc chọn Google Cloud project và cấu hình màn hình đồng ý OAuth, bao gồm test users nếu ứng dụng còn ở chế độ thử nghiệm.
2. Tạo OAuth client loại **Web application** cho luồng OAuth qua Supabase. Trong Authorized redirect URIs, dùng callback Supabase nêu trên, lấy chính xác từ màn hình cấu hình Google provider của Supabase.
3. Điền Client ID và Client Secret vào Google provider trong Supabase. Không đưa Client Secret vào Vue, Rust binary hoặc Git.

Nguồn: [Supabase Sign in with Google](https://supabase.com/docs/guides/auth/social-sign-in/auth-google).

### Apple Developer

1. Có quyền truy cập Apple Developer Program. Tạo hoặc chọn App ID có bật Sign in with Apple; xác nhận bundle ID dự kiến của TickTask (hiện là `com.ticktask.desktop` trong `src-tauri/tauri.conf.json`).
2. Tạo Services ID cho luồng web OAuth và liên kết với App ID. Trong Website URLs, domain là domain Supabase (`<project-ref>.supabase.co`), return URL là callback Supabase nêu trên.
3. Tạo signing key, giữ file `.p8` ở nơi bí mật, rồi tạo client secret theo hướng dẫn Supabase. Cấu hình Team ID, Services ID và secret trong Apple provider của Supabase. Không commit `.p8` hoặc client secret.
4. Lên lịch thay client secret **mỗi 6 tháng** nếu dùng luồng OAuth web. Mất hoặc lộ `.p8` thì thu hồi khóa và tạo khóa mới.

Apple có thể trả về địa chỉ email ẩn danh (Hide My Email). Không giả định email Apple luôn trùng email đăng nhập bằng mật khẩu/Google. Supabase tự liên kết các identity có cùng email theo quy tắc của nó; trường hợp email khác cần luồng liên kết tài khoản riêng nếu muốn dùng chung task.

Nguồn: [Supabase Sign in with Apple](https://supabase.com/docs/guides/auth/social-sign-in/auth-apple), [Apple Developer configuration](https://developer.apple.com/documentation/signinwithapple/configuring-your-environment-for-sign-in-with-apple), [Supabase identity linking](https://supabase.com/docs/guides/auth/auth-identity-linking).

## Các bước code trong TickTask

1. **Deep link và trình duyệt.** Thêm Tauri deep link và opener plugin, cấu hình desktop scheme trong `src-tauri/tauri.conf.json` và quyền trong `src-tauri/capabilities/default.json`. Bật tính năng `deep-link` cho `tauri-plugin-single-instance`; đăng ký single instance trước deep link plugin. Xử lý cả callback khi app đang mở và khi app được khởi động từ link. Trên Linux/Windows, kiểm tra trường hợp URL được chuyển qua tiến trình thứ hai. Xem [Tauri deep linking](https://v2.tauri.app/plugin/deep-linking/) và [Tauri opener](https://v2.tauri.app/plugin/opener/).
2. **Bắt đầu đăng nhập.** Thêm lệnh Tauri nhận provider chỉ cho phép `google` hoặc `apple`. Tạo PKCE verifier/challenge và giá trị state ngẫu nhiên, lưu tạm cho lần đăng nhập đang chờ, tạo URL Supabase Auth rồi mở bằng trình duyệt hệ thống. Không chấp nhận provider hoặc redirect URL tùy ý từ frontend.
3. **Hoàn tất callback.** Kiểm tra scheme, host/path, state, lỗi do người dùng từ chối và thời hạn của lần đăng nhập đang chờ. Trao đổi authorization code với Supabase bằng đúng PKCE verifier; chỉ chấp nhận phản hồi thành công và lấy thông tin người dùng từ Supabase trước khi lưu phiên. Không ghi URL callback, code hoặc token vào log.
4. **Gắn vào phiên hiện tại.** Dùng `AuthSession` và `TaskRepository::save_session` sẵn có; cập nhật trạng thái UI qua `taskStore.ts`, gọi chu kỳ sync như `login` hiện tại, tải lại danh sách task. Giữ nguyên luồng email/mật khẩu và đăng xuất.
5. **Giao diện.** Thêm nút “Tiếp tục với Google” và “Tiếp tục với Apple” trong `AuthModal.vue`; hiển thị trạng thái đang mở trình duyệt, chờ callback, lỗi hoặc hủy. Chặn bấm lặp tạo nhiều luồng PKCE chồng nhau.

**Lưu ý khi đổi tài khoản:** `save_session` hiện tự gán task khách (`user_id IS NULL`) cho tài khoản vừa đăng nhập. Kiểm tra dữ liệu khi chuyển giữa hai tài khoản và khi Apple dùng Hide My Email; tuyệt đối không tự gán task đã thuộc user A sang user B.

## Kiểm thử trước khi phát hành

- Google và Apple: đăng nhập mới, đăng nhập lại, hủy tại nhà cung cấp, callback lỗi, callback lặp hoặc hết hạn, đăng xuất và đăng nhập lại.
- Deep link: app đang mở, app đã đóng, bản cài đặt thật; kiểm thử riêng trên từng hệ điều hành được phát hành. Tauri cho biết desktop deep link thường cần ứng dụng đã cài đặt; môi trường dev trên Linux/Windows có thể cần đăng ký scheme thủ công.
- Dữ liệu: task khách được gán đúng user mới; task của user khác không lộ ra; đồng bộ chạy sau khi có phiên; refresh token vẫn hoạt động khi access token hết hạn.
- Tài khoản: email/mật khẩu hiện hữu đăng nhập tiếp được; thử Google cùng email, Apple cùng email, Apple Hide My Email. Kiểm tra hành vi liên kết identity thực tế trong Supabase trước khi coi các tài khoản là một.
- Bảo mật: callback giả hoặc state sai bị từ chối; token không xuất hiện trong URL, log hoặc lỗi hiển thị; secret của Google/Apple chỉ nằm trong bảng cấu hình nhà cung cấp.

## Thứ tự triển khai và điều kiện hoàn thành

Làm Google trước để hoàn thiện hạ tầng OAuth/PKCE và deep link; thêm Apple trên cùng hạ tầng sau khi cấu hình Developer account. Hoàn thành khi cả hai nút đăng nhập hoạt động trên bản cài đặt, phiên được khôi phục sau khi mở lại app, task đồng bộ đúng tài khoản và các trường hợp lỗi ở trên được xử lý rõ ràng.

Tài liệu này là kế hoạch triển khai; các provider, deep link và lệnh OAuth chưa được thêm vào ứng dụng.
