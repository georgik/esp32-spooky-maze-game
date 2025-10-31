use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rustc-link-arg=-Tlinkall.x");

    // Check if xtensa-esp32s3-elf-gcc is already in PATH
    if Command::new("xtensa-esp32s3-elf-gcc")
        .arg("--version")
        .output()
        .is_err()
    {
        // Toolchain not found in PATH, try to load from export-esp.sh
        setup_xtensa_environment();
    }
}

fn setup_xtensa_environment() {
    let home_dir = env::var("HOME").unwrap_or_else(|_| {
        env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string())
    });
    
    let export_script = PathBuf::from(&home_dir).join("export-esp.sh");
    
    if !export_script.exists() {
        eprintln!("Warning: {} not found. Please ensure Xtensa toolchain is in PATH.", export_script.display());
        return;
    }

    // Parse the export script to extract environment variables
    if let Ok(content) = std::fs::read_to_string(&export_script) {
        for line in content.lines() {
            let line = line.trim();
            
            // Handle LIBCLANG_PATH
            if line.starts_with("export LIBCLANG_PATH=") {
                if let Some(path) = extract_path_from_export(line, "LIBCLANG_PATH") {
                    println!("cargo:rustc-env=LIBCLANG_PATH={}", path);
                    unsafe { env::set_var("LIBCLANG_PATH", &path); }
                }
            }
            
            // Handle PATH
            if line.starts_with("export PATH=") {
                if let Some(path_addition) = extract_path_addition(line) {
                    let current_path = env::var("PATH").unwrap_or_default();
                    let new_path = if current_path.is_empty() {
                        path_addition
                    } else {
                        format!("{}:{}", path_addition, current_path)
                    };
                    println!("cargo:rustc-env=PATH={}", new_path);
                    unsafe { env::set_var("PATH", &new_path); }
                }
            }
        }
    }
}

fn extract_path_from_export(line: &str, var_name: &str) -> Option<String> {
    let prefix = format!("export {}=", var_name);
    if let Some(value) = line.strip_prefix(&prefix) {
        let value = value.trim_matches('"').trim_matches('\'');
        let expanded = expand_home(value);
        return Some(expanded);
    }
    None
}

fn extract_path_addition(line: &str) -> Option<String> {
    // Parse: export PATH="new_path:$PATH" or export PATH="new_path"
    if let Some(value) = line.strip_prefix("export PATH=") {
        let value = value.trim_matches('"').trim_matches('\'');
        
        // Extract just the new path additions (before $PATH)
        let parts: Vec<&str> = value.split('$').collect();
        if let Some(new_paths) = parts.first() {
            let paths = new_paths.trim_end_matches(':');
            let expanded = expand_home(paths);
            return Some(expanded);
        }
    }
    None
}

fn expand_home(path: &str) -> String {
    if path.starts_with("~/") {
        let home_dir = env::var("HOME").unwrap_or_else(|_| {
            env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string())
        });
        path.replacen("~", &home_dir, 1)
    } else {
        path.to_string()
    }
}
