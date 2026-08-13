use std::path::PathBuf;
fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_os != "windows" || target_env != "msvc" { return; }
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()));
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let target = std::env::var("TARGET").unwrap_or_default();
    let host = std::env::var("HOST").unwrap_or_default();
    let out_dir = if target == host || target.is_empty() { manifest_dir.join("target").join(&profile) } else { manifest_dir.join("target").join(&target).join(&profile) };
    let pkg_name = std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "rust_sqlite".to_string());
    println!("cargo:rustc-cdylib-link-arg=/OUT:{}", out_dir.join(format!("{}.xll", pkg_name)).display());
    println!("cargo:rerun-if-changed=build.rs");
}
