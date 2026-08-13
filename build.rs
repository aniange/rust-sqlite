use std::path::PathBuf;

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_os != "windows" || target_env != "msvc" {
        return;
    }
    if std::env::var("CARGO_CFG_TEST").is_ok() {
        return;
    }

    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()));
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let target = std::env::var("TARGET").unwrap_or_default();
    let host = std::env::var("HOST").unwrap_or_default();

    // 关键修复1：target == host 时 cargo 用 target/release/，cross-compile 时用 target/{target}/release/
    let out_dir = if target == host {
        manifest_dir.join("target").join(&profile)
    } else {
        manifest_dir.join("target").join(&target).join(&profile)
    };

    // 关键修复2：确保目录存在（cross-compile 时子目录可能未创建）
    std::fs::create_dir_all(&out_dir).unwrap();

    // 关键修复3：用 CARGO_CRATE_NAME（rust_sqlite）而非 CARGO_PKG_NAME（rust-sqlite）
    let crate_name =
        std::env::var("CARGO_CRATE_NAME").unwrap_or_else(|_| "rust_sqlite".to_string());

    println!(
        "cargo:rustc-cdylib-link-arg=/OUT:{}",
        out_dir.join(format!("{}.xll", crate_name)).display()
    );
    println!("cargo:rerun-if-changed=build.rs");
}
