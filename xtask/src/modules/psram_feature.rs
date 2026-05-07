use crate::modules::project::{ProjectInfo, TaskResult, TaskSummary};
use anyhow::Result;
use std::fs;

pub async fn remove_psram_feature(
    projects: &[ProjectInfo],
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    println!("\n[MIGRATE] Removing deprecated psram feature from esp-hal");
    if dry_run {
        println!("DRY-RUN mode - no changes will be made");
    }
    println!("{}", "=".repeat(60));

    let mut summary = TaskSummary::new();
    let mut results = Vec::new();

    for project in projects.iter().filter(|p| p.has_cargo_toml) {
        println!("\nProcessing: {}", project.name);

        let result = remove_project_psram_feature(project, dry_run, verbose).await?;

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
    println!("Migration Summary:");
    println!("Successfully processed: {} projects", summary.success);
    if summary.failed > 0 {
        println!("Failed: {} projects", summary.failed);
    }

    let migrated_count = results.iter().filter(|r| r.success && !r.message.is_empty()).count();
    if migrated_count > 0 {
        println!("\nProjects updated: {}", migrated_count);
        for result in results.iter().filter(|r| r.success && !r.message.is_empty()) {
            println!("  * {}", result.project);
        }
    }

    if migrated_count > 0 {
        println!("\nNext Steps:");
        if !dry_run {
            println!("1. Review updated Cargo.toml files");
            println!("2. Test builds: cargo xtask build");
        } else {
            println!("1. Review the above output");
            println!("2. Run without --dry-run to apply changes");
        }
    }

    Ok(())
}

async fn remove_project_psram_feature(
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

    // Remove psram feature from esp-hal dependency
    if new_content.contains("esp-hal") && new_content.contains("psram") {
        // Remove "psram" from features arrays
        new_content = new_content
            .replace("features = [\"esp32s3\", \"unstable\", \"psram\"]", "features = [\"esp32s3\", \"unstable\"]")
            .replace("features = [\"esp32\", \"unstable\", \"psram\"]", "features = [\"esp32\", \"unstable\"]")
            .replace("features = [\"esp32s3\", \"psram\"]", "features = [\"esp32s3\"]")
            .replace("esp-hal/psram", "")
            .replace("psram", "");

        // Clean up double commas and trailing commas
        new_content = new_content
            .replace(", ,", ",")
            .replace("[,", "[")
            .replace(", ]", "]")
            .replace("]  ", "]")
            .replace("  ]", "]");

        changes.push("Removed psram feature".to_string());
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
