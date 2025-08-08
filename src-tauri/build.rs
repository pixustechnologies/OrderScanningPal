use std::env;
use dotenvy::from_filename;

fn main() {

    let app_env = env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());

    // Build path to correct .env file
    let env_file = format!(".env.{}", app_env);

    // Load the environment file
    if let Err(err) = from_filename(&env_file) {
        println!("cargo:warning=Failed to load {}: {}", env_file, err);
    } else {
        println!("cargo:rerun-if-changed={}", env_file);
    }

    // from_filename(".env").ok();
    // println!("cargo:rerun-if-changed=.env");

    for key in ["DB_HOST", "DB_PORT", "DB_USER", "DB_PASSWORD", "DB_NAME", "DOC_PATH"] {
        if let Ok(val) = env::var(key) {
            println!("cargo:rustc-env={}={}", key, val);
        }
    }
    tauri_build::build()
}
