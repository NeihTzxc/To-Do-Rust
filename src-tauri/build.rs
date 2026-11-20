fn main() {
    // Tự động re-run build script nếu file .env thay đổi
    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-changed=../.env");

    // Nạp file .env từ thư mục hiện tại (src-tauri/.env) hoặc thư mục gốc dự án
    let _ = dotenvy::from_filename(".env")
        .or_else(|_| dotenvy::from_filename("../.env"))
        .or_else(|_| dotenvy::dotenv());

    // Nhúng biến môi trường vào binary lúc biên dịch bằng cargo:rustc-env
    if let Ok(url) = std::env::var("SUPABASE_URL") {
        if !url.trim().is_empty() {
            println!("cargo:rustc-env=SUPABASE_URL={}", url.trim());
        }
    }

    if let Ok(key) = std::env::var("SUPABASE_ANON_KEY") {
        if !key.trim().is_empty() {
            println!("cargo:rustc-env=SUPABASE_ANON_KEY={}", key.trim());
        }
    }

    tauri_build::build();
}