use crate::modules::project::{ProjectInfo, TaskResult, TaskSummary};
use anyhow::{Context, Result};
use std::fs;
use tokio::process::Command as TokioCommand;

pub async fn build_all_projects(
    projects: &[ProjectInfo],
    keep_going: bool,
    verbose: bool,
) -> Result<()> {
    println!("\n[BUILD] Building all ESP32 projects with --release profile");
    println!("{}", "=".repeat(60));

    let mut summary = TaskSummary::new();
    let mut results = Vec::new();

    for project in projects.iter().filter(|p| p.has_cargo_toml) {
        println!("\nBuilding: {}", project.name);

        let result = build_project(project, verbose).await?;

        if result.success {
            println!("OK: {}", project.name);
            if !result.warnings.is_empty() {
                println!("Warning: {} warnings found:", result.warnings.len());
                for warning in &result.warnings[..std::cmp::min(5, result.warnings.len())] {
                    println!("   {}", warning);
                }
                if result.warnings.len() > 5 {
                    println!("   ... and {} more warnings", result.warnings.len() - 5);
                }
            }
        } else {
            println!("FAILED: {}", project.name);
            println!("Error details:");
            for line in result.message.lines().take(20) {
                println!("   {}", line);
            }
            if result.message.lines().count() > 20 {
                println!("   ... (truncated, use --verbose for full output)");
            }
            if !keep_going {
                return Err(anyhow::anyhow!("Build failed for {}", project.name));
            }
        }

        summary.add_result(&result);
        results.push(result);
    }

    print_build_summary(&summary, &results).await;

    if summary.failed > 0 && !keep_going {
        std::process::exit(1);
    }

    Ok(())
}

async fn build_project(project: &ProjectInfo, verbose: bool) -> Result<TaskResult> {
    let toolchain_path = project.path.join("rust-toolchain.toml");
    let toolchain = if toolchain_path.exists() {
        let content =
            fs::read_to_string(&toolchain_path).context("Failed to read rust-toolchain.toml")?;

        if content.contains("channel = \"esp\"") {
            "esp"
        } else if content.contains("channel = \"stable\"") {
            "stable"
        } else {
            "stable"
        }
    } else {
        "stable"
    };

    let output = TokioCommand::new("rustup")
        .arg("run")
        .arg(toolchain)
        .arg("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(&project.path)
        .output()
        .await
        .context("Failed to run cargo build")?;

    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut warnings = Vec::new();
    let combined_output = format!("{}{}", stdout, stderr);

    for line in combined_output.lines() {
        if verbose {
            println!("   {}", line);
        }
        if line.contains("warning:") {
            warnings.push(line.to_string());
        }
    }

    let message = if success {
        "Built successfully".to_string()
    } else {
        format!("{}{}", stdout, stderr)
    };

    Ok(TaskResult {
        project: project.name.clone(),
        success,
        message,
        warnings,
    })
}

async fn print_build_summary(summary: &TaskSummary, results: &[TaskResult]) {
    println!("\n{}", "=".repeat(60));
    println!("Build Summary:");
    println!("Successfully built: {} projects", summary.success);
    if summary.failed > 0 {
        println!("Failed to build: {} projects", summary.failed);
        println!("\nFailed projects:");
        for result in results.iter().filter(|r| !r.success) {
            println!("  * {}", result.project);
        }
    }
    if summary.warnings > 0 {
        println!("Total warnings: {}", summary.warnings);
        println!("\nProjects with warnings:");
        for result in results.iter().filter(|r| !r.warnings.is_empty()) {
            println!(
                "  * {} ({} warnings)",
                result.project,
                result.warnings.len()
            );
        }
    }

    if summary.failed == 0 && summary.warnings == 0 {
        println!("All projects built successfully with no warnings!");
    } else if summary.failed == 0 {
        println!("All projects built successfully (with some warnings)");
    }
}

pub async fn format_all_projects(projects: &[ProjectInfo], verbose: bool) -> Result<()> {
    println!("\n[FORMAT] Formatting all ESP32 projects");
    println!("{}", "=".repeat(60));

    let mut summary = TaskSummary::new();

    for project in projects.iter().filter(|p| p.has_cargo_toml) {
        println!("\nFormatting: {}", project.name);

        let result = format_project(project, verbose).await?;

        if result.success {
            println!("OK: {}", project.name);
        } else {
            println!("FAILED: {}", project.name);
            if verbose {
                println!("Error: {}", result.message);
            }
        }

        summary.add_result(&result);
    }

    println!("\n{}", "=".repeat(60));
    println!("Format Summary:");
    println!("Successfully formatted: {} projects", summary.success);
    if summary.failed > 0 {
        println!("Failed: {} projects", summary.failed);
    }

    Ok(())
}

async fn format_project(project: &ProjectInfo, verbose: bool) -> Result<TaskResult> {
    let toolchain_path = project.path.join("rust-toolchain.toml");
    let toolchain = if toolchain_path.exists() {
        let content =
            fs::read_to_string(&toolchain_path).context("Failed to read rust-toolchain.toml")?;

        if content.contains("channel = \"esp\"") {
            "esp"
        } else if content.contains("channel = \"stable\"") {
            "stable"
        } else {
            "stable"
        }
    } else {
        "stable"
    };

    let output = TokioCommand::new("rustup")
        .arg("run")
        .arg(toolchain)
        .arg("cargo")
        .arg("fmt")
        .current_dir(&project.path)
        .output()
        .await
        .context("Failed to run cargo fmt")?;

    let success = output.status.success();
    let stderr = String::from_utf8_lossy(&output.stderr);

    if verbose && !stderr.is_empty() {
        println!("   {}", stderr);
    }

    Ok(TaskResult {
        project: project.name.clone(),
        success,
        message: if success {
            "Formatted".to_string()
        } else {
            stderr.to_string()
        },
        warnings: Vec::new(),
    })
}
