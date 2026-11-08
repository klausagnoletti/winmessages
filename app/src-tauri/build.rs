fn main() {
    // unicorn (QEMU) uses 16-byte atomics that need libatomic on Linux
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=atomic");
    tauri_build::build()
}
