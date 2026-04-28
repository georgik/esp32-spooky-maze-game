use crate::modules::project::{ProjectInfo, TaskResult, TaskSummary};
use anyhow::Result;
use std::fs;

pub async fn clean_deprecated_config(
    projects: &[ProjectInfo],
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    println!("\n[CLEAN] Removing deprecated build configuration options");
    if dry_run {
        println!("DRY-RUN mode - no changes will be made");
    }
    println!("{}", "=".repeat(60));

    let mut summary = TaskSummary::new();
    let mut results = Vec::new();

    for project in projects.iter().filter(|p| p.has_cargo_toml) {
        println!("\nProcessing: {}", project.name);

        let result = clean_project_config(project, dry_run, verbose).await?;

        if result.success {
            if !result.message.is_empty() {
                println!("Cleaned: {}", project.name);
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
    println!("Config Cleanup Summary:");
    println!("Successfully cleaned: {} projects", summary.success);
    if summary.failed > 0 {
        println!("Failed: {} projects", summary.failed);
    }

    let changed_count = results.iter().filter(|r| r.success && !r.message.is_empty()).count();
    if changed_count > 0 {
        println!("\nProjects with changes: {}", changed_count);
        for result in results.iter().filter(|r| r.success && !r.message.is_empty()) {
            println!("  * {}", result.project);
        }
    }

    if changed_count > 0 {
        println!("\nNext Steps:");
        if !dry_run {
            println!("1. Review updated .cargo/config.toml files");
            println!("2. Test builds: cargo xtask build");
            println!("3. Verify functionality of updated projects");
        } else {
            println!("1. Review the above output");
            println!("2. Run without --dry-run to apply changes");
        }
    }

    Ok(())
}

async fn clean_project_config(
    project: &ProjectInfo,
    dry_run: bool,
    _verbose: bool,
) -> Result<TaskResult> {
    let cargo_config_path = project.path.join(".cargo").join("config.toml");

    if !cargo_config_path.exists() {
        return Ok(TaskResult {
            project: project.name.clone(),
            success: true,
            message: String::new(),
            warnings: Vec::new(),
        });
    }

    let content = match fs::read_to_string(&cargo_config_path) {
        Ok(content) => content,
        Err(e) => {
            return Ok(TaskResult {
                project: project.name.clone(),
                success: false,
                message: format!("Failed to read config.toml: {}", e),
                warnings: Vec::new(),
            });
        }
    };

    let mut changes = Vec::new();
    let mut new_content = content.clone();

    let deprecated_options = &[
        "ESP_HAL_CONFIG_PSRAM_MODE",
        "ESP_HAL_CONFIG_XTAL_FREQUENCY",
    ];

    for option in deprecated_options {
        if new_content.contains(option) {
            let lines: Vec<&str> = new_content.lines().collect();
            let filtered_lines: Vec<&str> = lines
                .iter()
                .filter(|line| !line.contains(option))
                .map(|&line| line)
                .collect();

            new_content = filtered_lines.join("\n");

            if !new_content.is_empty() && !new_content.ends_with('\n') {
                new_content.push('\n');
            }

            changes.push(format!("Removed {}", option));
        }
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
        if let Err(e) = fs::write(&cargo_config_path, &new_content) {
            return Ok(TaskResult {
                project: project.name.clone(),
                success: false,
                message: format!("Failed to write config.toml: {}", e),
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

pub async fn fix_workspace_issues(projects: &[ProjectInfo]) -> Result<()> {
    println!("\n[FIX] Fixing workspace issues in all ESP32 projects");
    println!("{}", "=".repeat(60));

    let mut fixed_count = 0;
    let mut skipped_count = 0;

    for project in projects.iter().filter(|p| p.has_cargo_toml) {
        let cargo_toml_path = project.path.join("Cargo.toml");

        println!("\nProcessing: {}", project.name);

        let content = match fs::read_to_string(&cargo_toml_path) {
            Ok(content) => content,
            Err(e) => {
                println!("FAILED: Failed to read Cargo.toml: {}", e);
                continue;
            }
        };

        if content.contains("[workspace]") {
            println!("SKIPPED: Already has [workspace] section");
            skipped_count += 1;
            continue;
        }

        let lines: Vec<&str> = content.lines().collect();
        let mut new_lines = Vec::new();
        let mut added_workspace = false;

        for line in lines {
            new_lines.push(line.to_string());

            if !added_workspace && line.starts_with("[") && !line.starts_with("[package]") {
                new_lines.insert(new_lines.len() - 1, String::new());
                new_lines.insert(
                    new_lines.len() - 1,
                    "# Empty workspace to avoid being part of parent workspace".to_string(),
                );
                new_lines.insert(new_lines.len() - 1, "[workspace]".to_string());
                new_lines.insert(new_lines.len() - 1, String::new());
                added_workspace = true;
            }
        }

        if !added_workspace {
            new_lines.push(String::new());
            new_lines.push("# Empty workspace to avoid being part of parent workspace".to_string());
            new_lines.push("[workspace]".to_string());
            new_lines.push(String::new());
        }

        let new_content = new_lines.join("\n");

        match fs::write(&cargo_toml_path, new_content) {
            Ok(_) => {
                println!("OK: Added empty [workspace] section");
                fixed_count += 1;
            }
            Err(e) => {
                println!("FAILED: Failed to write Cargo.toml: {}", e);
            }
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("Workspace Fix Summary:");
    println!("Fixed: {} projects", fixed_count);
    if skipped_count > 0 {
        println!(
            "Skipped: {} projects (already had [workspace])",
            skipped_count
        );
    }

    if fixed_count > 0 {
        println!("\nAll projects should now build independently!");
        println!("Try: cargo xtask build");
    }

    Ok(())
}
