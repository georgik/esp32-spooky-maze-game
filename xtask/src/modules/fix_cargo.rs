use crate::modules::project::{ProjectInfo, TaskResult, TaskSummary};
use anyhow::Result;
use std::fs;

pub async fn fix_corrupted_cargo_toml(
    projects: &[ProjectInfo],
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    println!("\n[FIX] Fixing corrupted Cargo.toml files");
    if dry_run {
        println!("DRY-RUN mode - no changes will be made");
    }
    println!("{}", "=".repeat(60));

    let mut summary = TaskSummary::new();
    let mut results = Vec::new();

    for project in projects.iter().filter(|p| p.has_cargo_toml) {
        println!("\nProcessing: {}", project.name);

        let result = fix_project_cargo_toml(project, dry_run, verbose).await?;

        if result.success {
            if !result.message.is_empty() {
                println!("OK: {}", project.name);
                if verbose {
                    println!("Changes: {}", result.message);
                }
            } else {
                if verbose {
                    println!("OK: No changes needed: {}", project.name);
                }
            }
        } else {
            println!("FAILED: {}", project.name);
            if !result.message.is_empty() {
                println!("Error: {}", result.message);
            }
        }

        summary.add_result(&result);
        results.push(result);
    }

    println!("\n{}", "=".repeat(60));
    println!("Fix Summary:");
    println!("Successfully processed: {} projects", summary.success);
    if summary.failed > 0 {
        println!("Failed: {} projects", summary.failed);
    }

    let fixed_count = results.iter().filter(|r| r.success && !r.message.is_empty()).count();
    if fixed_count > 0 {
        println!("\nProjects fixed: {}", fixed_count);
        for result in results.iter().filter(|r| r.success && !r.message.is_empty()) {
            println!("  * {}", result.project);
        }
    }

    Ok(())
}

pub async fn scan_corrupted_files(projects: &[ProjectInfo]) -> Result<Vec<String>> {
    println!("\n[SCAN] Scanning for corrupted Cargo.toml files");
    println!("{}", "=".repeat(60));

    let mut corrupted = Vec::new();

    for project in projects.iter().filter(|p| p.has_cargo_toml) {
        let cargo_toml_path = project.path.join("Cargo.toml");

        if !cargo_toml_path.exists() {
            continue;
        }

        match fs::read_to_string(&cargo_toml_path) {
            Ok(content) => {
                if content.contains("\"\"") || content.contains("[,") || content.contains(",]") {
                    corrupted.push(project.name.clone());
                    println!("CORRUPTED: {}", project.name);
                }
            }
            Err(e) => {
                println!("ERROR reading {}: {}", project.name, e);
            }
        }
    }

    println!("\n{}", "=".repeat(60));
    if corrupted.is_empty() {
        println!("No corrupted files found");
    } else {
        println!("Found {} corrupted files", corrupted.len());
        for name in &corrupted {
            println!("  * {}", name);
        }
    }

    Ok(corrupted)
}

async fn fix_project_cargo_toml(
    project: &ProjectInfo,
    dry_run: bool,
    _verbose: bool,
) -> Result<TaskResult> {
    let cargo_toml_path = project.path.join("Cargo.toml");

    if !cargo_toml_path.exists() {
        return Ok(TaskResult {
            project: project.name.clone(),
            success: true,
            message: String::new(),
            warnings: Vec::new(),
        });
    }

    let content = match fs::read_to_string(&cargo_toml_path) {
        Ok(content) => content,
        Err(e) => {
            return Ok(TaskResult {
                project: project.name.clone(),
                success: false,
                message: format!("Failed to read Cargo.toml: {}", e),
                warnings: Vec::new(),
            });
        }
    };

    let mut changes = Vec::new();
    let mut new_content = content.clone();

    // Fix corrupted patterns from aggressive psram removal
    if new_content.contains("\"\"") {
        // Remove empty strings from features arrays
        new_content = new_content
            .replace(", \"\"", ",")
            .replace("[ \"\"]", "[]")
            .replace("[\"\"]", "[]")
            .replace("\"\"]", "\"")
            .replace(", \"\"", "")
            .replace("[ \"\",", "[")
            .replace("[\",", "[")
            .replace("  ]", "]");

        changes.push("Fixed empty strings in features".to_string());
    }

    // Clean up double commas and trailing commas
    if new_content.contains(", ,") || new_content.contains("[,") || new_content.contains(",]") {
        new_content = new_content
            .replace(", ,", ",")
            .replace("[,", "[")
            .replace(",]", "]")
            .replace("]  ", "]")
            .replace("  ]", "]");

        changes.push("Fixed comma issues".to_string());
    }

    if changes.is_empty() {
        return Ok(TaskResult {
            project: project.name.clone(),
            success: true,
            message: String::new(),
            warnings: Vec::new(),
        });
    }

    if !dry_run {
        if let Err(e) = fs::write(&cargo_toml_path, &new_content) {
            return Ok(TaskResult {
                project: project.name.clone(),
                success: false,
                message: format!("Failed to write Cargo.toml: {}", e),
                warnings: Vec::new(),
            });
        }
    }

    Ok(TaskResult {
        project: project.name.clone(),
        success: true,
        message: changes.join(", "),
        warnings: Vec::new(),
    })
}

pub async fn scan_and_fix_corrupted_files(
    projects: &[ProjectInfo],
    dry_run: bool,
) -> Result<()> {
    println!("\n[SCAN & FIX] Scanning and fixing corrupted Cargo.toml files");
    if dry_run {
        println!("DRY-RUN mode - no changes will be made");
    }
    println!("{}", "=".repeat(60));

    let mut corrupted_projects = Vec::new();

    // First pass: scan for corrupted files
    for project in projects.iter().filter(|p| p.has_cargo_toml) {
        let cargo_toml_path = project.path.join("Cargo.toml");

        match fs::read_to_string(&cargo_toml_path) {
            Ok(content) => {
                if is_corrupted_cargo_toml(&content) {
                    corrupted_projects.push(project.name.clone());
                    println!("CORRUPTED: {}", project.name);
                }
            }
            Err(e) => {
                println!("ERROR reading {}: {}", project.name, e);
            }
        }
    }

    if corrupted_projects.is_empty() {
        println!("\nNo corrupted files found");
        return Ok(());
    }

    println!("\nFound {} corrupted files", corrupted_projects.len());

    // Second pass: fix corrupted files
    for project in projects.iter().filter(|p| p.has_cargo_toml) {
        if !corrupted_projects.contains(&project.name) {
            continue;
        }

        println!("\nFixing: {}", project.name);

        let cargo_toml_path = project.path.join("Cargo.toml");
        let content = match fs::read_to_string(&cargo_toml_path) {
            Ok(content) => content,
            Err(e) => {
                println!("FAILED: Could not read: {}", e);
                continue;
            }
        };

        let fixed_content = fix_corrupted_content(&content);

        if fixed_content != content {
            if !dry_run {
                match fs::write(&cargo_toml_path, &fixed_content) {
                    Ok(_) => println!("OK: Fixed"),
                    Err(e) => println!("FAILED: Could not write: {}", e),
                }
            } else {
                println!("DRY-RUN: Would fix");
            }
        } else {
            println!("SKIP: No changes needed");
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("Scan & Fix Complete");
    if !dry_run {
        println!("Next: Run 'cargo xtask scan-cargo' to verify fixes");
    }

    Ok(())
}

fn is_corrupted_cargo_toml(content: &str) -> bool {
    content.contains("\"\"") ||
    content.contains("[,") ||
    content.contains(",]") ||
    content.contains(", ,") ||
    content.contains("authors = [") && !content.contains("authors = [\"")
}

fn fix_corrupted_content(content: &str) -> String {
    let mut new_content = content.to_string();

    // Fix corrupted authors field: [Name " <email>""]
    new_content = new_content
        .replace(r#"authors = [Juraj Michálek " <juraj.michalek@espressif.com>""]"#,
                 r#"authors = ["Juraj Michálek <juraj.michalek@espressif.com>"]"#);

    // Fix authors field: missing quotes around array content
    if new_content.contains("authors = [") && !new_content.contains("authors = [\"") {
        new_content = new_content
            .replace("authors = [", "authors = [\"")
            .replace("<", "\" <")
            .replace(">", ">\"");
    }

    // Fix missing opening quotes in esp-hal features: features = [esp32s3", "unstable"]
    let chip_patterns = [
        ("esp32\"", "\"esp32\""),
        ("esp32c3\"", "\"esp32c3\""),
        ("esp32c6\"", "\"esp32c6\""),
        ("esp32s2\"", "\"esp32s2\""),
        ("esp32s3\"", "\"esp32s3\""),
        ("esp32h2\"", "\"esp32h2\""),
    ];

    for (corrupted, fixed) in &chip_patterns {
        new_content = new_content.replace(corrupted, fixed);
    }

    // Fix cases where both opening and closing quotes are missing: features = [esp32s3", ...]
    let chip_opening = [
        ("features = [esp32", "features = [\"esp32"),
        ("features = [esp32c3", "features = [\"esp32c3"),
        ("features = [esp32c6", "features = [\"esp32c6"),
        ("features = [esp32s2", "features = [\"esp32s2"),
        ("features = [esp32s3", "features = [\"esp32s3"),
        ("features = [esp32h2", "features = [\"esp32h2"),
    ];

    for (corrupted, fixed) in &chip_opening {
        new_content = new_content.replace(corrupted, fixed);
    }

    // Fix features like: [esp32s3""] (double quote at end)
    new_content = new_content
        .replace("esp32\"]", "esp32\"]")
        .replace("esp32c3\"]", "esp32c3\"]")
        .replace("esp32c6\"]", "esp32c6\"]")
        .replace("esp32s2\"]", "esp32s2\"]")
        .replace("esp32s3\"]", "esp32s3\"]")
        .replace("esp32h2\"]", "esp32h2\"]");

    // Fix empty strings in features
    new_content = new_content
        .replace(", \"\"", ",")
        .replace("[ \"\"]", "[]")
        .replace("[\"\"]", "[]")
        .replace("\"\" ]", "]")
        .replace("[ \"\"", "[")
        .replace("[\"", "[");

    // Fix comma issues
    new_content = new_content
        .replace(", ,", ",")
        .replace("[,", "[")
        .replace(",]", "]");

    // Clean up extra spaces
    new_content = new_content
        .replace("  ]", "]")
        .replace("]  ", "]")
        .replace("  ,", ",")
        .replace("  [", "[")
        .replace("[  ", "[")
        .replace("}  ", "}")
        .replace("  }", "}");

    new_content
}
