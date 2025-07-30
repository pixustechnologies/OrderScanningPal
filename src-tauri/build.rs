use std::env;
use dotenvy::from_filename;

fn main() {
    from_filename(".env").ok();
    println!("cargo:rerun-if-changed=.env");

    for key in ["DB_HOST", "DB_PORT", "DB_USER", "DB_PASSWORD", "DB_NAME"] {
        if let Ok(val) = env::var(key) {
            println!("cargo:rustc-env={}={}", key, val);
        }
    }
    tauri_build::build()
}
