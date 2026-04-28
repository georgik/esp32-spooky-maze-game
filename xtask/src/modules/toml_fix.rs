use crate::modules::project::{ProjectInfo, TaskResult, TaskSummary};
use anyhow::Result;
use std::fs;
use toml::Value;

pub async fn fix_cargo_toml_quotes(
    projects: &[ProjectInfo],
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    println!("\n[FIX] Fixing missing quotes in Cargo.toml features");
    if dry_run {
        println!("DRY-RUN mode - no changes will be made");
    }
    println!("{}", "=".repeat(60));

    let mut summary = TaskSummary::new();
    let mut results = Vec::new();

    for project in projects.iter().filter(|p| p.has_cargo_toml) {
        println!("\nProcessing: {}", project.name);

        let result = fix_project_quotes(project, dry_run, verbose).await?;

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

    if fixed_count > 0 {
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

async fn fix_project_quotes(
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

    // Parse TOML and fix feature arrays
    match content.parse::<Value>() {
        Ok(mut toml_value) => {
            let modified = fix_feature_quotes(&mut toml_value);
            if modified {
                changes.push("Fixed missing quotes in feature arrays".to_string());
            }

            if !changes.is_empty() && !dry_run {
                let new_content = toml::to_string(&toml_value)?;
                if let Err(e) = fs::write(&cargo_toml_path, new_content) {
                    return Ok(TaskResult {
                        project: project.name.clone(),
                        success: false,
                        message: format!("Failed to write Cargo.toml: {}", e),
                        warnings: Vec::new(),
                    });
                }
            }
        }
        Err(e) => {
            return Ok(TaskResult {
                project: project.name.clone(),
                success: false,
                message: format!("Failed to parse TOML: {}", e),
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

fn fix_feature_quotes(toml_value: &mut Value) -> bool {
    let mut modified = false;

    // Fix dependencies section
    if let Some(deps) = toml_value.get_mut("dependencies") {
        if let Some(deps_table) = deps.as_table_mut() {
            for (_, dep_value) in deps_table.iter_mut() {
                if let Some(dep) = dep_value.as_table_mut() {
                    if let Some(features) = dep.get_mut("features") {
                        if let Some(feat_array) = features.as_array_mut() {
                            for feature in feat_array.iter_mut() {
                                if let Some(feat_str) = feature.as_str() {
                                    if !feat_str.starts_with('"') && !feat_str.contains("->") {
                                        if let Some(new_str) = format!("\"{}\"", feat_str).parse::<Value>().ok() {
                                            *feature = new_str;
                                            modified = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Fix features section
    if let Some(feat_section) = toml_value.get_mut("features") {
        if let Some(feat_table) = feat_section.as_table_mut() {
            for (_, feat_value) in feat_table.iter_mut() {
                if let Some(feat_array) = feat_value.as_array_mut() {
                    for feature in feat_array.iter_mut() {
                        if let Some(feat_str) = feature.as_str() {
                            // Fix esp-hal/esp32 style references
                            if feat_str.contains("esp-hal/") && !feat_str.contains("\"esp-hal/") {
                                if let Some(new_str) = feat_str.replace("esp-hal/", "\"esp-hal/").parse::<Value>().ok() {
                                    *feature = new_str;
                                    modified = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    modified
}
